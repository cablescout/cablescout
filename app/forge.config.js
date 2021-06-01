module.exports = {
  packagerConfig: {
    name: 'Cablescout',
    out: './out',
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
        prerelease: true,
      },
    },
  ],
}
