//! PKIX is deliberately bypassed for wire compatibility; CertificateVerify and
//! TLS 1.2 key-exchange signatures are always checked by ring/webpki. Application
//! bytes remain gated by the reactor's pin trust state after handshake completion.
use crate::{identity::failed, Fingerprint, Identity, NetError};
use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider},
    pki_types::{CertificateDer, ServerName, UnixTime},
    server::danger::{ClientCertVerified, ClientCertVerifier},
    ClientConfig, Connection, DigitallySignedStruct, DistinguishedName, Error, ServerConfig, SignatureScheme,
};
use std::sync::Arc;

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
    fn sig12(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls12_signature(msg, cert, dss, &self.provider.signature_verification_algorithms)
    }
    fn sig13(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls13_signature(msg, cert, dss, &self.provider.signature_verification_algorithms)
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
