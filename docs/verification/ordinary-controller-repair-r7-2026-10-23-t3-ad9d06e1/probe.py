"""One real read-only snapshot, bounded by the caller and its own alarm. Never run in this task."""
from common import *
from macproc import snapshot, targeted
import sys


def main():
    nice15()
    deadline = float(sys.argv[1])
    with io_until(deadline):
        if sys.argv[3:4] == ['--targeted']:
            result = targeted(sys.argv[4:])
        else:
            tokens = sys.argv[3:]
            split = tokens.index('--private-candidates') if '--private-candidates' in tokens else len(tokens)
            result = snapshot(Path(sys.argv[2]).resolve(), tokens[:split], tokens[split + 1:])
        print(json.dumps(result, separators=(',', ':')))


if __name__ == '__main__':
    main()
