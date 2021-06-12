import * as path from 'path'
import log from 'electron-log'
import prompt from 'electron-prompt'
import { getStatus } from './client'
import { runAsRoot } from './sudo'
import { updateTray } from './tray'

export async function addTunnel(): Promise<void> {
    let url: string | null = null
    try {
        url = await prompt({
            title: 'Add Tunnel',
            label: 'Please enter tunnel URL:',
            inputAttrs: {
                type: 'url',
                required: 'true',
            },
            type: 'input',
        })
    } catch (err) {
        log.error(`[add-tunnel] Error getting tunnel URL: ${err}`)
    }
    if (!url) {
        log.warn('[add-tunnel] User cancelled')
        return
    }
    await createTunnelFile(url)
}

async function createTunnelFile(endpoint: string): Promise<void> {
    log.info(`[add-tunnel] Adding new tunnel ${endpoint}`)
    const tunnel_info = {
        endpoint,
    }
    const parsed_url = new URL('', endpoint)
    const name = parsed_url.host

    const status = await getStatus()
    const tunnels_path = status.tunnelsPath
    if (!tunnels_path) {
        const msg = `Can't get tunnels path from status: ${JSON.stringify(status)}`
        log.error(`[add-tunnel] ${msg}`)
        throw new Error(msg)
    }

    if (status.config && status.config[name]) {
        const msg = `Tunnel ${endpoint} already exists`
        log.error(`[add-tunnel] ${msg}`)
        throw new Error(msg)
    }

    const tunnel_file = path.join(tunnels_path, `${name}.tunnel.json`)
    await runAsRoot(`echo '${JSON.stringify(tunnel_info)}' > "${tunnel_file}"`)
    await updateTray()
}
