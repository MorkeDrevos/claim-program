name: Deploy Anchor (devnet)

on:
  workflow_dispatch:
  push:
    branches: [ "main" ]
    paths:
      - "programs/**"
      - "Anchor.toml"
      - ".github/workflows/deploy-devnet.yml"

jobs:
  build-deploy:
    runs-on: ubuntu-latest

    permissions:
      contents: write  # needed if you want to commit PROGRAM_ID.devnet

    env:
      SOLANA_VERSION: "1.18.22"
      ANCHOR_CLI_VERSION: "0.29.0"
      CLUSTER: "devnet"

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      # ---- RUST TOOLCHAIN 1.76.0 (FIX rustc 1.75.0-dev) ----
      - name: Install Rust 1.76
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: 1.76.0

      - name: Show rustc version
        run: rustc --version

      # ---- SOLANA CLI ----
      - name: Install Solana CLI (tarball)
        shell: bash
        run: |
          set -euo pipefail
          curl -sSL -o solana.tar.bz2 "https://github.com/solana-labs/solana/releases/download/v${SOLANA_VERSION}/solana-release-x86_64-unknown-linux-gnu.tar.bz2"
          mkdir -p "$HOME/solana-release"
          tar -xjf solana.tar.bz2 --strip-components=1 -C "$HOME/solana-release"
          echo "$HOME/solana-release/bin" >> "$GITHUB_PATH"
          solana --version

      - name: Install Anchor CLI
        run: |
          npm install -g @coral-xyz/anchor-cli@${ANCHOR_CLI_VERSION}
          anchor --version

      - name: Install jq (for IDL export)
        run: sudo apt-get update && sudo apt-get install -y jq

      - name: Configure keypair & RPC
        shell: bash
        run: |
          set -euo pipefail
          mkdir -p "$HOME/.config/solana"
          echo '${{ secrets.SOLANA_KEYPAIR_JSON }}' > "$HOME/.config/solana/id.json"
          chmod 600 "$HOME/.config/solana/id.json"

          RPC="${{ secrets.SOLANA_RPC_URL }}"
          if [ -z "$RPC" ]; then
            RPC="https://api.devnet.solana.com"
          fi

          solana config set --url "$RPC"
          solana address

      # 🚫 IMPORTANT: we REMOVED the "Sanitize Cargo.toml" step here
      # so your [patch.crates-io] section with toml_edit = "=0.23.2"
      # and toml_parser = "=1.0.3" stays in place.

      - name: Build
        shell: bash
        run: |
          set -euo pipefail
          rustc --version
          cargo clean || true
          ANCHOR_NO_CONFIG_WARN=1 anchor build

      - name: Deploy (devnet)
        shell: bash
        run: |
          set -euo pipefail
          anchor deploy --provider.cluster "${CLUSTER}"
          if [ -f target/deploy/claim-keypair.json ]; then
            PROGRAM_ID=$(solana address -k target/deploy/claim-keypair.json)
            echo "PROGRAM_ID=${PROGRAM_ID}" | tee program-id.txt
          fi

      - name: Export IDL
        shell: bash
        run: |
          set -euo pipefail
          cp target/idl/claim.json idl.json

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: claim-idl-and-program
          path: |
            program-id.txt
            idl.json

      - name: Commit program id (optional)
        if: always()
        shell: bash
        run: |
          if [ -f program-id.txt ]; then
            PROGRAM_ID=$(cat program-id.txt)
            echo "$PROGRAM_ID" > PROGRAM_ID.devnet
            git config user.name "github-actions[bot]"
            git config user.email "github-actions[bot]@users.noreply.github.com"
            git add PROGRAM_ID.devnet || true
            git commit -m "chore: update devnet program id -> $PROGRAM_ID" || echo "no changes"
            git push || true
          else
            echo "program-id.txt missing — skipping commit."
          fi
