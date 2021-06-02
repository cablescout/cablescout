module.exports = {
  packagerConfig: {
    name: 'Cablescout',
    out: './out',
    extraResource: [
      '../target/release/cablescout-daemon',
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
