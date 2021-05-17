const axios = require('axios')
const { to_ini } = require('./stupid-ini')
const { wgKeyPair, wgQuickUp, wgQuickDown } = require('./wg')
const { oauthLogin } = require('./login')
const { STATUS } = require('./status')

class Tunnel {
  constructor(name, config) {
    this.name = name
    this.config = config
  }

  startApiUrl() {
    const url = new URL('/api/v1/login/start', this.config.endpoint)
    return url.toString()
  }

  finishApiUrl() {
    const url = new URL('/api/v1/login/finish', this.config.endpoint)
    return url.toString()
  }

  finishUrl() {
    const url = new URL('/finish', this.config.endpoint)
    return url.toString()
  }

  async login() {
    const key_pair = await wgKeyPair()

    const start_res = await axios.post(this.startApiUrl(), {
      client_public_key: key_pair.public_key,
    })

    const auth_code = await oauthLogin(start_res.data.auth_url, this.finishUrl())

    const finish_res = await axios.post(this.finishApiUrl(), {
      login_token: start_res.data.login_token,
      auth_code,
    })

    const { session_ends_at } = finish_res.data

    return {
      Interface: {
        PrivateKey: key_pair.private_key,
        ...finish_res.data.interface,
      },
      Peer: finish_res.data.peer,
    }
  }

  async connect() {
    const config = await this.login()
    const config_ini = to_ini(config)
    await wgQuickUp(this.name, config_ini)
    STATUS.setConnected(true)
  }

  async disconnect() {
    await wgQuickDown(this.name)
    STATUS.setConnected(false)
  }
}

module.exports = {
  Tunnel,
}
