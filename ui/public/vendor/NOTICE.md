# Vendored third-party libraries

These files are bundled locally (not loaded from a CDN — the app's Content-Security-Policy is
`script-src 'self'`) rather than hand-implemented, because a spec-perfect QR codec is easy to get
subtly wrong in ways that pass self-testing but fail on real-world scanners. Both were verified
with an independent round-trip test (encode with `qrcode`, decode the result with `jsQR`) before
being vendored here.

- **qrcode.js** — [node-qrcode](https://github.com/soldair/node-qrcode) by Ryan Day, MIT License.
  Bundled from npm `qrcode` package's browser entry point (`lib/browser.js`) via esbuild.
- **jsQR.js** — [jsQR](https://github.com/cozmo/jsQR) by cozmo, Apache-2.0 License. Used as
  published (`dist/jsQR.js`).
