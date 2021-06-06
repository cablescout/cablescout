const process = require('process')

const DAEMON_BASENAME = (process.platform === 'win32') ? 'cablescout-daemon.exe' : 'cablescout-daemon'
const { APPLE_API_KEY, APPLE_API_ISSUER } = process.env

module.exports = {
  packagerConfig: {
    name: 'Cablescout',
    out: './out',
    extraResource: [
      `../target/release/${DAEMON_BASENAME}`,
    ],
    osxSign: {
      "hardened-runtime": true,
      "entitlements": "entitlements.plist",
      "entitlements-inherit": "entitlements.plist",
      "signature-flags": "library"
    },
    osxNotarize: {
      appleApiKey: APPLE_API_KEY,
      appleApiIssuer: APPLE_API_ISSUER,
    },
  },

  makers: [
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin', 'win32'],
    },
  ],

  publishers: [
    {
      name: '@electron-forge/publisher-github',
      config: {
        repository: {
          owner: 'cablescout',
          name: 'cablescout',
        },
      },
    },
  ],
}
