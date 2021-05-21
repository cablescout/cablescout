import * as os from 'os'
import { mkdir, readFile, stat, writeFile } from 'fs/promises'
import * as path from 'path'
import log from 'electron-log'
import * as yaml from 'js-yaml'
import { TunnelConfig } from './tunnel'

const HOME = os.homedir()
const CONFIG_DIR = path.join(HOME, '.cablescout')
const TUNNELS_FILE = path.join(CONFIG_DIR, 'tunnels.yaml')

type AllTunnels = Record<string, TunnelConfig>

class Config {
    async getTunnels(): Promise<AllTunnels> {
        log.debug('[config] Reading tunnels from config file')
        try {
            await stat(TUNNELS_FILE)
        } catch (err) {
            return {}
        }
        const raw = await readFile(TUNNELS_FILE)
        const tunnels = yaml.load(raw.toString()) as AllTunnels
        log.debug(`[config] Read config file: ${JSON.stringify(tunnels, null, 2)}`)
        return tunnels
    }

    async writeTunnels(tunnels: AllTunnels) {
        log.debug(`[config] Writing config file: ${JSON.stringify(tunnels, null, 2)}`)
        await mkdir(CONFIG_DIR, { recursive: true })
        const raw = yaml.dump(tunnels)
        await writeFile(TUNNELS_FILE, raw)
    }

    async addTunnel(name: string, tunnel: TunnelConfig) {
        log.debug(`[config] Adding tunnel: name=${name} tunnel=${JSON.stringify(tunnel)}`)
        const tunnels = await this.getTunnels()
        tunnels[name] = tunnel
        await this.writeTunnels(tunnels)
    }
}

export const CONFIG = new Config()
