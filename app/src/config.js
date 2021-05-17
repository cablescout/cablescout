const os = require('os')
const { mkdir, readFile, stat, writeFile } = require('fs/promises')
const path = require('path')
const yaml = require('js-yaml')

const HOME = os.homedir()
const CONFIG_DIR = path.join(HOME, '.cablescout')
const TUNNELS_FILE = path.join(CONFIG_DIR, 'tunnels.yaml')

class Config {
  async getTunnels() {
    try {
      await stat(TUNNELS_FILE)
    } catch (err) {
      return {}
    }
    const raw = await readFile(TUNNELS_FILE)
    return yaml.load(raw)
  }

  async writeTunnels(tunnels) {
    await mkdir(CONFIG_DIR, { recursive: true })
    const raw = yaml.dump(tunnels)
    await writeFile(TUNNELS_FILE, raw)
  }

  async addTunnel(name, tunnel) {
    const tunnels = await this.get_tunnels()
    tunnels[name] = tunnel
    await this.write_tunnels(tunnels)
  }
}

module.exports = {
  CONFIG: new Config(),
}
