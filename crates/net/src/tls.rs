//! PKIX is deliberately bypassed for wire compatibility; CertificateVerify and
//! TLS 1.2 key-exchange signatures are always checked by ring. Application
//! bytes remain gated by the reactor's pin trust state after handshake completion.
use crate::{identity::failed, Fingerprint, Identity, NetError};
use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{verify_tls13_signature_with_raw_key, CryptoProvider},
    pki_types::{CertificateDer, ServerName, SubjectPublicKeyInfoDer, UnixTime},
    server::danger::{ClientCertVerified, ClientCertVerifier},
    CertificateError, ClientConfig, Connection, DigitallySignedStruct, DistinguishedName, Error, PeerMisbehaved,
    ServerConfig, SignatureScheme,
};
use std::sync::Arc;

/// Splits one DER element into (tag, contents, rest).
fn der(input: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, input) = input.split_first()?;
    let (&first, input) = input.split_first()?;
    let (len, input) = match first {
        0..=0x7f => (first as usize, input),
        0x81..=0x84 => {
            let n = (first & 0x7f) as usize;
            let (bytes, rest) = (input.get(..n)?, input.get(n..)?);
            (bytes.iter().fold(0usize, |acc, b| acc << 8 | *b as usize), rest)
        }
        _ => return None,
    };
    Some((tag, input.get(..len)?, input.get(len..)?))
}

/// The whole DER SubjectPublicKeyInfo element of an X.509 certificate of any version.
fn spki(cert: &[u8]) -> Option<&[u8]> {
    let (0x30, cert, _) = der(cert)? else { return None };
    let (0x30, tbs, _) = der(cert)? else { return None };
    let mut rest = tbs;
    if rest.first() == Some(&0xa0) {
        rest = der(rest)?.2; // explicit version, absent in v1
    }
    for _ in 0..5 {
        rest = der(rest)?.2; // serialNumber, signature, issuer, validity, subject
    }
    let (0x30, _, after) = der(rest)? else { return None };
    Some(&rest[..rest.len() - after.len()])
}

/// (AlgorithmIdentifier contents, subjectPublicKey bits) of a SubjectPublicKeyInfo.
fn spki_parts(spki: &[u8]) -> Option<(&[u8], &[u8])> {
    let (0x30, spki, _) = der(spki)? else { return None };
    let (0x30, alg_id, rest) = der(spki)? else { return None };
    let (0x03, bits, _) = der(rest)? else { return None };
    let (&0, key) = bits.split_first()? else { return None };
    Some((alg_id, key))
}

#[derive(Debug)]
struct PinVerifier {
    provider: Arc<CryptoProvider>,
    expected: Option<Fingerprint>,
    require_client: bool,
}
impl PinVerifier {
    fn pin(&self, der: &CertificateDer<'_>) -> Result<(), Error> {
        if self
            .expected
            .is_some_and(|fp| fp != Fingerprint::from_der(der.as_ref()))
        {
            return Err(Error::General("certificate fingerprint mismatch".into()));
        }
        Ok(())
    }
    // Signatures are checked against the certificate's SubjectPublicKeyInfo rather than a
    // webpki end-entity parse, which rejects X.509 v1. Peers whose certificate came from
    // LibreSSL's `openssl req -x509` (Barrier, older Synergy on macOS) send v1 certificates.
    fn sig12(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        let (alg_id, key) = spki(cert.as_ref())
            .and_then(spki_parts)
            .ok_or(Error::InvalidCertificate(CertificateError::BadEncoding))?;
        let algorithms = &self.provider.signature_verification_algorithms;
        let candidates = algorithms
            .mapping
            .iter()
            .find(|(scheme, _)| *scheme == dss.scheme)
            .map(|(_, algs)| *algs)
            .ok_or(PeerMisbehaved::SignedHandshakeWithUnadvertisedSigScheme)?;
        let alg = candidates
            .iter()
            .find(|alg| alg.public_key_alg_id().as_ref() == alg_id)
            .ok_or(Error::InvalidCertificate(CertificateError::BadSignature))?;
        alg.verify_signature(key, msg, dss.signature())
            .map(|_| HandshakeSignatureValid::assertion())
            .map_err(|_| Error::InvalidCertificate(CertificateError::BadSignature))
    }
    fn sig13(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        let spki = spki(cert.as_ref()).ok_or(Error::InvalidCertificate(CertificateError::BadEncoding))?;
        verify_tls13_signature_with_raw_key(
            msg,
            &SubjectPublicKeyInfoDer::from(spki),
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }
    fn schemes(&self) -> Vec<SignatureScheme> {
        self.provider.signature_verification_algorithms.supported_schemes()
    }
}
impl ServerCertVerifier for PinVerifier {
    fn verify_server_cert(
        &self,
        cert: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        self.pin(cert)?;
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.sig12(msg, cert, dss)
    }
    fn verify_tls13_signature(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.sig13(msg, cert, dss)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.schemes()
    }
}
impl ClientCertVerifier for PinVerifier {
    fn client_auth_mandatory(&self) -> bool {
        self.require_client
    }
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }
    fn verify_client_cert(
        &self,
        cert: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: UnixTime,
    ) -> Result<ClientCertVerified, Error> {
        self.pin(cert)?;
        Ok(ClientCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.sig12(msg, cert, dss)
    }
    fn verify_tls13_signature(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.sig13(msg, cert, dss)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.schemes()
    }
}

pub(crate) struct TlsFactory {
    identity: Identity,
    provider: Arc<CryptoProvider>,
    server: Arc<ServerConfig>,
}
impl TlsFactory {
    pub(crate) fn new(identity: Identity, require_client: bool) -> Result<Self, NetError> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let verifier = Arc::new(PinVerifier {
            provider: provider.clone(),
            expected: None,
            require_client,
        });
        let server = ServerConfig::builder_with_provider(provider.clone())
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .map_err(failed)?
            .with_client_cert_verifier(verifier)
            .with_single_cert(identity.chain.clone(), identity.key.clone_key())
            .map_err(failed)?;
        Ok(Self {
            identity,
            provider,
            server: Arc::new(server),
        })
    }
    pub(crate) fn server(&self) -> Result<Connection, NetError> {
        let mut conn = Connection::Server(rustls::ServerConnection::new(self.server.clone()).map_err(failed)?);
        conn.set_buffer_limit(Some(64 * 1024));
        Ok(conn)
    }
    pub(crate) fn client(&self, host: &str, expected: Option<Fingerprint>) -> Result<Connection, NetError> {
        let verifier = Arc::new(PinVerifier {
            provider: self.provider.clone(),
            expected,
            require_client: true,
        });
        let config = ClientConfig::builder_with_provider(self.provider.clone())
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .map_err(failed)?
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_client_auth_cert(self.identity.chain.clone(), self.identity.key.clone_key())
            .map_err(failed)?;
        let server_name = ServerName::try_from(host.to_owned()).map_err(failed)?;
        let mut conn =
            Connection::Client(rustls::ClientConnection::new(Arc::new(config), server_name).map_err(failed)?);
        conn.set_buffer_limit(Some(64 * 1024));
        Ok(conn)
    }
}
