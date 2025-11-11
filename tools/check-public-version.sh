#!/bin/bash

here=$(dirname $(realpath $0))
root=$(dirname $here)

local_version=$(cat $root/yamake/Cargo.toml | grep "^version")
local_major=$(($(echo $local_version | sed "s/.*\"\(.*\)\.\(.*\)\.\(.*\)\"/\1/")))
local_minor=$(($(echo $local_version | sed "s/.*\"\(.*\)\.\(.*\)\.\(.*\)\"/\2/")))
local_micro=$(($(echo $local_version | sed "s/.*\"\(.*\)\.\(.*\)\.\(.*\)\"/\3/")))

# echo $local_version
# echo $local_major
# echo $local_minor
# echo $local_micro

published_version=$(cargo show yamake | yq ".max_version")
published_major=$(($(echo $published_version | sed "s/\(.*\)\.\(.*\)\.\(.*\)/\1/")))
published_minor=$(($(echo $published_version | sed "s/\(.*\)\.\(.*\)\.\(.*\)/\2/")))
published_micro=$(($(echo $published_version | sed "s/\(.*\)\.\(.*\)\.\(.*\)/\3/")))


# echo $published_version
# echo $published_major
# echo $published_minor
# echo $published_micro

if ! [[ $local_major -eq $published_major ]] ; then
    printf "bad major\n"
    exit 1
fi

if ! [[ $local_minor -eq $published_minor ]] ; then
    printf "bad minor\n"
    exit 1
fi

expected=$((published_micro + 1))
if [[ $expected -ne $local_micro ]] ; then
    printf "local micro is $local_micro, should be $expected\n"
    exit 1
fi

printf "version ok\n"
exit 0
