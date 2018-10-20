#/usr/bin/python3
#fuck fucking useless shit `make`

import os
import re
import sys
import subprocess

VERSION = "v0-1"#None

LAST_STATUS = None
LAST_OUTPUT = None

def fail(msg):
    print(msg)
    sys.exit(1)

def exec(cmd):
    global LAST_STATUS
    global LAST_OUTPUT

    res = subprocess.run(cmd, shell=True)
    LAST_STATUS = res.returncode
    # FIXME: assign res.stdout to `LAST_OUTPUT`
    #LAST_OUTPUT = res.stdout #+ res.stderr
    return LAST_STATUS == 0 

# FIXME: check if a the specified version was covered
# should also include 0.5 in [0.1, 1.0], because 
# it is between
def version_is_covered(version):
    return version in get_available_versions() 

def get_available_versions():
    # FIXME: make this relative or set `$PWD`
    # FIXME: buffer this 
    versions = []
    with open("Cargo.toml") as f:
        it = iter(f.readlines())
        for line in it:
            if '[features]' in line:
                break
        for line in it:
            if not re.match(r"v\d+-\d+.*", line):
                break
            versions.append(line.split('=')[0].strip())
    # FIXME: sort versions; could help using version_is_covered
    return versions

def is_valid_version(version):
    return re.match(r"v\d+-\d+", version)

def compile_features(to_version):
    for version in get_available_versions():
        if version == to_version:
            break
        yield version
    yield to_version

def cargo_test():
    global VERSION

    features = compile_features(VERSION)
    return exec("cargo test --features '%s'" % ' '.join(features))

def main():
    global VERSION

    print(list(get_available_versions()))

    if len(sys.argv) < 2:
        fail("`VERSION` must be specified")

    VERSION = sys.argv[1]
    if not is_valid_version(sys.argv[1]):
        fail("%s is not a valid version" % VERSION)

    if not cargo_test():
        print(LAST_OUTPUT)
        fail("test failed.")

    # FIXME: if there isn't already a `VERSION` git-tag: create it

if __name__ == '__main__':
    main()
