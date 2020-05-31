#!/bin/bash
set -eo pipefail

: "${DRY_RUN:?"DRY_RUN has to be specified"}"
if [[ "$DRY_RUN" != "false" ]]; then
    exit 0
fi

#: "${CRATE_NAME:?"CRATE_NAME has to be specified"}"
#: "${CRATE_ROOT:?"CRATE_ROOT has to be specified"}"
: "${NEW_VERSION:?"NEW_VERSION has to be specified"}"
#: "${PREV_VERSION:?"PREV_VERSION has to be specified"}"
#: "${WORKSPACE_ROOT:?"WORKSPACE_ROOT has to be specified"}"

if [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    IS_PRERELEASE=false
else
    IS_PRERELEASE=true
fi
function is_prerelease {
    [[ "$IS_PRERELEASE" == "true" ]]
}

function _replace {
    local marker="$1"
    local usage="$2"

    local tempfile
    tempfile="$(mktemp)"
    {
        echo "<!-- $marker -->"
        echo "\`\`\`text"
        echo "$usage"
        echo "\`\`\`"
        echo "<!-- /$marker -->"
    } >> "$tempfile"

    sed \
        -i \
        -e "/<!-- $marker -->/,/<!-- \/$marker -->/!b" \
        -e "/<!-- \/$marker -->/!d;r $tempfile" \
        -e "d" \
        README.md

    rm "$tempfile"
}
function readme_replace_usage {
    if is_prerelease; then
        return 0
    fi

    _replace "usage-main" "$(cargo run -- --help)"
    _replace "usage-layout" "$(cargo run -- layout --help)"
}

readme_replace_usage
