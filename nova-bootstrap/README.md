# nova-bootstrap

A standalone, headless bootstrap/relay peer for the NOVA Chat DHT. It is not a cloud service that
owns user data: it never sees plaintext (relayed traffic is already end-to-end ciphertext) and
DHT records are self-authenticating, so it does not need to be trusted — it only needs to be
reachable. Anyone can run one; it doesn't have to be the app's own developers.

## What it's for

Two devices with no prior contact (e.g. one in Ouagadougou, one in Bobo-Dioulasso) have no way to
find each other on the internet without *some* known rendezvous point — this is true of every
P2P network (BitTorrent, IPFS, Tox), not a shortcut specific to this app. A `nova-bootstrap`
instance is that rendezvous point: clients dial it once to join the Kademlia DHT, and — since
every NOVA node also runs circuit-relay-v2 — it can relay traffic between two peers who can't
reach each other directly (e.g. both behind carrier-grade NAT, common on West African mobile
networks).

## Running locally

```bash
cargo run -p nova-bootstrap --release
```

On first run this generates a 12-word identity and saves it to `bootstrap_identity.txt` in the
working directory — keep this file, or the node's address changes every restart and nobody can
keep pointing at it. It prints the address to hand to clients, then idles until Ctrl+C.

## Testing across 2–3 physical devices (no VPS)

For validating the app with real hardware instead of a cloud host, two setups cover it, in order
of how much you need to configure.

### Same Wi-Fi network — zero configuration

If all test devices are on the same LAN (e.g. everyone's laptop/phone on the same home or office
Wi-Fi), **no bootstrap node is needed at all**. `P2PNode` runs mDNS (`dht_node.rs`) and
auto-discovers/dials any other NOVA peer on the local network within a couple of seconds of
launch. Just build and run `nova-desktop` on each machine:

```bash
cargo tauri dev   # or: cargo tauri build, then run the produced binary
```

This is the setup to validate first — it exercises the full real stack (X3DH, Double Ratchet,
QUIC transport, message persistence, voice/location/file transfer) with zero network config, and
isolates whether a problem is in the app logic versus in cross-network reachability.

### Devices on different networks — one device acts as the rendezvous point

To mirror a real deployment (two people on different ISPs/mobile networks, no shared LAN), one of
your physical test devices plays the role otherwise played by a VPS. Any always-on-for-the-test
machine works — a laptop is fine for a test session, it does not need to be a server. Use the
standalone `nova-bootstrap` binary for this role rather than a second `nova-desktop` instance: it
prints the exact multiaddr (including the libp2p peer ID) the other devices need to copy, which
`nova-desktop` does not currently surface in its UI.

1. **On the chosen device (call it A), run `nova-bootstrap`:**

   ```bash
   cargo run -p nova-bootstrap --release
   ```

   It generates a 12-word identity on first run (saved to `bootstrap_identity.txt` — keep it, or
   the address changes next restart) and prints something like:

   ```
   give this to a client for first contact:
   /ip4/192.168.1.23/udp/4001/quic-v1/p2p/12D3KooW...
   ```

   That address is A's *local* IP — not reachable from another network yet.

2. **Make port UDP `4001` reachable from outside A's network.** Log into A's router admin page
   (typically `192.168.1.1` or `192.168.0.1`) and forward external UDP `4001` to A's LAN IP and
   port `4001`. If A is on a mobile/CGNAT connection this will not work — in that case run
   `nova-bootstrap` on whichever available device has a normal home/office broadband connection
   instead (CGNAT is why a plain "run it on your phone's hotspot" setup fails; it isn't specific
   to this app).

3. **Find A's public IP** (from A itself: open a browser and search "what is my ip", or run
   `curl.exe ifconfig.me` in PowerShell), then restart `nova-bootstrap` with it set explicitly —
   auto-detection cannot reliably tell a private interface IP from the public one:

   ```powershell
   $env:NOVA_BOOTSTRAP_PUBLIC_ADDR = "/ip4/<A-PUBLIC-IP>/udp/4001/quic-v1"
   cargo run -p nova-bootstrap --release
   ```

   The identity file makes the peer ID stable across this restart. It now prints:

   ```
   give this to a client for first contact (NOVA_BOOTSTRAP_PUBLIC_ADDR):
   /ip4/<A-PUBLIC-IP>/udp/4001/quic-v1/p2p/12D3KooW...
   ```

   **Copy that whole line** — it already has the correct peer ID baked in.

4. **On the other devices (B, and C if used), launch `nova-desktop` with that address:**

   ```powershell
   $env:NOVA_BOOTSTRAP_ADDR = "/ip4/<A-PUBLIC-IP>/udp/4001/quic-v1/p2p/12D3KooW..."
   cargo tauri dev
   ```

   This is only needed once per app launch, for first contact — it is what lets that device's
   node join the same Kademlia DHT as A and become discoverable/dialable by peer ID afterward.
   Set `RUST_LOG=info` in the same session beforehand if you want to watch the dial succeed in
   the terminal.

5. **Exchange identities and chat.** On each device's Identity screen, share the prekey bundle
   (copy/paste, or the QR code) and add each other as a contact on the other device, then send a
   message. The Diagnostics screen's latency reading is now the real measured dial/round-trip
   time over this link (expect tens to low hundreds of ms, not the ~0–2 ms you'd see over
   localhost) — not the local-proxy figure it used to show.

A can double as one of the two chatting devices — nothing stops it from also running its own
`nova-desktop` instance at the same time as `nova-bootstrap` (two separate processes, two
separate identities, no port conflict since `nova-desktop` defaults to a random port unless
`NOVA_LISTEN_ADDR` is set). A third device (C) just repeats step 4–5 against the same address.

If none of the available devices can accept inbound UDP at all (e.g. everyone is behind mobile
CGNAT), a cross-network test isn't possible without *some* externally reachable node — that's the
one case where falling back to a cheap always-on host for just the rendezvous role (not for
message storage; it never sees plaintext) is the pragmatic option, per the VPS section below.

## Deploying on a real VPS (optional — only if no test device can accept inbound traffic)

1. Build a release binary (either on the VPS itself, or cross-compiled):

   ```bash
   cargo build -p nova-bootstrap --release
   ```

   The binary is at `target/release/nova-bootstrap`.

2. Open the UDP port you intend to use (default `4001`) in the VPS firewall / cloud provider's
   security group. This is UDP, not TCP — QUIC runs over UDP.

3. Set `NOVA_BOOTSTRAP_PUBLIC_ADDR` to the VPS's actual public IP. **Do not skip this on any host
   with more than one network interface** (common on VPS providers with a private + public NIC,
   or anywhere running Docker/a VPN) — a device cannot reliably learn its own public IP by
   enumerating its own interfaces, so auto-detection may pick the wrong one silently.

4. Run it under a process supervisor so it survives reboots/crashes. Example systemd unit
   (`/etc/systemd/system/nova-bootstrap.service`):

   ```ini
   [Unit]
   Description=NOVA Chat DHT bootstrap/relay node
   After=network.target

   [Service]
   Type=simple
   User=nova-bootstrap
   WorkingDirectory=/opt/nova-bootstrap
   Environment=NOVA_BOOTSTRAP_LISTEN=/ip4/0.0.0.0/udp/4001/quic-v1
   Environment=NOVA_BOOTSTRAP_PUBLIC_ADDR=/ip4/YOUR.PUBLIC.IP.HERE/udp/4001/quic-v1
   ExecStart=/opt/nova-bootstrap/nova-bootstrap
   Restart=on-failure
   RestartSec=5

   [Install]
   WantedBy=multi-user.target
   ```

   ```bash
   sudo useradd --system --home /opt/nova-bootstrap --shell /usr/sbin/nologin nova-bootstrap
   sudo mkdir -p /opt/nova-bootstrap
   sudo cp target/release/nova-bootstrap /opt/nova-bootstrap/
   sudo chown -R nova-bootstrap:nova-bootstrap /opt/nova-bootstrap
   sudo systemctl daemon-reload
   sudo systemctl enable --now nova-bootstrap
   sudo journalctl -u nova-bootstrap -f   # watch it start; note the printed address
   ```

5. Take the address it prints (with `NOVA_BOOTSTRAP_PUBLIC_ADDR` set, this is
   `<public-addr>/p2p/<libp2p-peer-id>`) and hand it to clients as their bootstrap address —
   this is what a client passes to `P2PNode::bootstrap_dial(...)` on first contact.

## Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `NOVA_BOOTSTRAP_LISTEN` | `/ip4/0.0.0.0/udp/4001/quic-v1` | Local multiaddr to bind. |
| `NOVA_BOOTSTRAP_PUBLIC_ADDR` | unset | The address to advertise to clients — set this explicitly in production (see above). |
| `NOVA_BOOTSTRAP_IDENTITY_FILE` | `bootstrap_identity.txt` | Where the node's mnemonic is persisted across restarts. |
| `RUST_LOG` | unset | Standard `tracing` filter, e.g. `RUST_LOG=info` or `RUST_LOG=nova_transport=debug`. |

## Scaling beyond one node

Nothing here requires exactly one bootstrap node. Running several (at different operators, on
different networks) is strictly better for resilience — clients only need to know the address of
*one* to join the DHT, after which discovery is distributed across whoever's actually storing
each record. There is no reason this needs to be centralized long-term.
