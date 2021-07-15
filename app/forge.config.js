const process = require('process')

const BUNDLE_ID = 'io.cablescout'
const DAEMON_BASENAME = (process.platform === 'win32') ? 'cablescout-daemon.exe' : 'cablescout-daemon'

const { APPLE_API_KEY, APPLE_API_ISSUER } = process.env
const osxNotarize = (process.platform === 'darwin') ? {
  appBundleId: BUNDLE_ID,
  appleApiKey: APPLE_API_KEY,
  appleApiIssuer: APPLE_API_ISSUER,
} : undefined

const extraResource = [
  `../target/release/${DAEMON_BASENAME}`,
]

if (process.platform === 'win32') {
  extraResource.push('out/shawl.exe')
}

module.exports = {
  packagerConfig: {
    name: 'Cablescout',
    appBundleId: BUNDLE_ID,
    out: './out',
    extraResource,
    osxSign: {
      "hardened-runtime": true,
      "entitlements": "entitlements.plist",
      "entitlements-inherit": "entitlements.plist",
      "signature-flags": "library"
    },
    osxNotarize,
  },

  makers: [
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin'],
    },
    {
      name: '@electron-forge/maker-squirrel',
      platforms: ['win32'],
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
