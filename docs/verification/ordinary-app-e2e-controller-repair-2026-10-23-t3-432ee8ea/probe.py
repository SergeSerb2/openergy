"""One real read-only snapshot, bounded by the caller and its own alarm. Never run in this task."""
from common import *
from macproc import snapshot, targeted
import sys


def main():
    nice15()
    deadline = float(sys.argv[1])
    with io_until(deadline):
        result = (targeted(sys.argv[4:]) if sys.argv[3:4] == ['--targeted'] else
                  snapshot(Path(sys.argv[2]).resolve(), sys.argv[3:]))
        print(json.dumps(result, separators=(',', ':')))


if __name__ == '__main__':
    main()
