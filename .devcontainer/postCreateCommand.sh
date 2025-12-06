#!/usr/bin/env zsh

set -exo pipefail

if ! grep -q mise ~/.zshrc; then
    # shellcheck disable=SC2016
    echo 'eval "$(mise activate zsh)"' >> ~/.zshrc
fi

mise trust
mise install -y
eval "$(mise activate zsh)"

npm install -g aws-cdk

cat <<'EOF' >> ~/.zshrc
export AWS_DEFAULT_PROFILE='manage'

complete -C '/usr/local/bin/aws_completer' aws
EOF

source ~/.zshrc
