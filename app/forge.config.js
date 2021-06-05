const process = require('process')

const DAEMON_BASENAME = (process.platform === 'win32') ? 'cablescout-daemon.exe' : 'cablescout-daemon'

module.exports = {
  packagerConfig: {
    name: 'Cablescout',
    out: './out',
    extraResource: [
      `../target/release/${DAEMON_BASENAME}`,
    ],
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
