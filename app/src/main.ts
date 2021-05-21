import * as path from 'path'
import { app, Menu, Tray } from 'electron'
import log from 'electron-log'
import { CONFIG } from './config'
import { STATUS } from './status'
import { Tunnel, TunnelConfig } from './tunnel'

const TRAY_ICON_OFF = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.Off.Template.png')
const TRAY_ICON_PROGRESS = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.Progress.Template.png')
const TRAY_ICON_ON = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.On.Template.png')

let tray: Tray = null

async function appWillQuit(event: Event) {
    log.warn('[main] App about to quit')
    if (!STATUS.isConnected()) {
        log.info('[main] No tunnel connected, quitting app')
        return
    }
    event.preventDefault()
    const tunnel_name = STATUS.currTunnel()
    log.info(`[main] Disconnecting ${tunnel_name} before quitting app`)
    const tunnels = await CONFIG.getTunnels()
    const tunnel_config = tunnels[tunnel_name]
    if (tunnel_config) {
        await disconnectTunnel(tunnel_name, tunnel_config)
    }
    app.quit()
}

async function connectTunnel(name: string, config: TunnelConfig) {
    try {
        STATUS.setCurrTunnel(name)
        updateTray()
        const tunnel = new Tunnel(name, config)
        await tunnel.connect()
    } catch (err) {
        log.error(`[main] error: ${err}`)
        STATUS.setCurrTunnel(null)
    }
    updateTray()
}

async function disconnectTunnel(name: string, config: TunnelConfig) {
    const tunnel = new Tunnel(name, config)
    await tunnel.disconnect()
    STATUS.setCurrTunnel(null)
    updateTray()
}

async function updateTray() {
    log.debug('[main] Updating tray icon')
    const tunnels = await CONFIG.getTunnels()

    const current_tunnel = STATUS.currTunnel()
    log.debug(`[main] Current tunnel: ${current_tunnel}`)

    const tunnel_menu_items = Object.entries(tunnels).map(([name, tunnel_config]) => {
        const is_curr = current_tunnel === name
        return {
            label: is_curr ? `Disconnect ${name}` : `Connect ${name}`,
            click: is_curr ? (() => disconnectTunnel(name, tunnel_config)) : (() => connectTunnel(name, tunnel_config))
        }
    })

    const menu = Menu.buildFromTemplate([
        ...tunnel_menu_items,
        {
            label: 'Add new tunnel...',
            enabled: false,
        },
        {
            type: 'separator',
        },
        {
            label: 'About Cablescout',
            enabled: false,
        },
        {
            label: 'Quit',
            click() {
                app.quit()
            },
        },
    ])

    if (!tray) {
        tray = new Tray(TRAY_ICON_OFF)

        // error TS2769: No overload matches this call.
        // Argument of type '"click"' is not assignable to parameter of type '"right-click"'.
        //if (process.platform === 'win32') {
        //    tray.on('click', tray.popUpContextMenu)
        //}
    }

    tray.setImage(STATUS.currTunnel() ? (STATUS.isConnected() ? TRAY_ICON_ON : TRAY_ICON_PROGRESS) : TRAY_ICON_OFF)
    tray.setContextMenu(menu)
}

async function main() {
    log.info('[main] Starting app')
    await app.whenReady()
    log.debug('[main] App ready')

    if (app.dock) {
        app.dock.hide()
    }

    app.on('window-all-closed', (event: Event) => event.preventDefault())
    app.on('will-quit', appWillQuit)

    await updateTray()
}

log.catchErrors({ showDialog: true })
main()
