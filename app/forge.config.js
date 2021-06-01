const { env } = require('process')

module.exports = {
  packagerConfig: {
    name: 'Cablescout',
    out: './out',
  },

  makers: [
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin', 'win'],
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
        prerelease: true,
        authToken: env.GITHUB_TOKEN,
      },
    },
  ],
}
