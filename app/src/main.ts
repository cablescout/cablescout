import * as path from 'path'
import { app, dialog, Menu, Tray } from 'electron'
import log from 'electron-log'
import { CONFIG } from './config'
import { STATUS } from './status'
import { Tunnel, TunnelConfig } from './tunnel'

const TRAY_ICON_OFF = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.Off.Template.png')
const TRAY_ICON_PROGRESS = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.Progress.Template.png')
const TRAY_ICON_ON = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.On.Template.png')

let tray: Tray | null = null

async function appWillQuit(event: Event) {
    log.warn('[main] App about to quit')
    const curr_tunnel = STATUS.currTunnel()
    if (!curr_tunnel) {
        log.info('[main] No tunnel connected, quitting app')
        return
    }
    event.preventDefault()
    log.info(`[main] Disconnecting ${curr_tunnel.name} before quitting app`)
    await curr_tunnel.disconnect()
    app.quit()
}

async function connectTunnel(name: string, config: TunnelConfig) {
    try {
        const tunnel = new Tunnel(name, config)
        STATUS.setCurrTunnel(tunnel)
        updateTray()
        await tunnel.connect()
    } catch (err) {
        log.error(`[main] error: ${err}`)
        dialog.showErrorBox(`Error connecting to ${name}`, err)
        STATUS.setCurrTunnel(undefined)
    }
    updateTray()
}

async function disconnectTunnel(tunnel: Tunnel) {
    await tunnel.disconnect()
    STATUS.setCurrTunnel(undefined)
    updateTray()
}

async function updateTray() {
    log.debug('[main] Updating tray icon')
    const tunnels = await CONFIG.getTunnels()

    const curr_tunnel = STATUS.currTunnel()
    log.debug(`[main] Current tunnel: ${curr_tunnel ? curr_tunnel.name : 'null'}`)

    const tunnel_menu_items = Object.entries(tunnels).map(
        ([name, tunnel_config]) => (curr_tunnel && (curr_tunnel.name === name)) ? {
            label: `Disconnect ${name}`,
            click: () => disconnectTunnel(curr_tunnel),
        } : {
            label: `Connect ${name}`,
            click: () => connectTunnel(name, tunnel_config),
        }
    )

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

    tray.setImage(curr_tunnel ? (curr_tunnel.isConnected() ? TRAY_ICON_ON : TRAY_ICON_PROGRESS) : TRAY_ICON_OFF)
    tray.setContextMenu(menu)
}

async function main() {
    log.info('[main] =================== Starting app ===================')
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
