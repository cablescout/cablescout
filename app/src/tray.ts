import * as path from 'path'
import log from 'electron-log'
import { app, Menu, Tray } from 'electron'
import { TunnelStatus } from '../proto-gen/daemon_api/TunnelStatus'
import { TunnelInfo } from '../proto-gen/daemon_api/TunnelInfo'
import { getStatus, connectTunnel, disconnectTunnel } from './client'
import { addTunnel } from './add-tunnel'
import { StatusResponse } from '../proto-gen/daemon_api/StatusResponse'

const TRAY_ICON_OFF = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.Off.Template.png')
const TRAY_ICON_PROGRESS = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.Progress.Template.png')
const TRAY_ICON_ON = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.On.Template.png')
const TRAY_ICON_ERROR = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.Error.Template.png')

let tray: Tray | null = null

export async function updateTray(): Promise<void> {
    log.debug('[main] Updating tray icon')
    try {
        const status = await getStatus()
        updateTrayFromStatus(status)
    } catch (error) {
        updateTrayFromStatus({
            currentTunnel: {
                status: TunnelStatus.ERROR,
            },
        } as StatusResponse)
        throw error
    }
}

export function updateTrayFromStatus(status: StatusResponse): void {
    if (!tray) {
        tray = new Tray(TRAY_ICON_OFF)

        // error TS2769: No overload matches this call.
        // Argument of type '"click"' is not assignable to parameter of type '"right-click"'.
        //if (process.platform === 'win32') {
        //    tray.on('click', tray.popUpContextMenu)
        //}
    }

    const curr_tunnel = status.currentTunnel
    log.debug(`[main] Current tunnel: ${curr_tunnel?.name}`)

    const tunnel_menu_items = status.config ? Object.keys(status.config as Record<string, TunnelInfo>).map(
        (name) => (curr_tunnel?.name === name) ? {
            label: `Disconnect ${name}`,
            click: () => disconnectTunnel(),
        } : {
            label: `Connect ${name}`,
            click: () => connectTunnel(name),
        }
    ) : []

    const menu = Menu.buildFromTemplate([
        ...tunnel_menu_items,
        {
            label: 'Add new tunnel...',
            click: addTunnel,
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

    switch (curr_tunnel?.status) {
        case TunnelStatus.CONNECTED:
            tray.setImage(TRAY_ICON_ON)
            break
        case TunnelStatus.CONNECTING:
        case TunnelStatus.DISCONNECTING:
            tray.setImage(TRAY_ICON_PROGRESS)
            break
        case TunnelStatus.ERROR:
            tray.setImage(TRAY_ICON_ERROR)
            break
        default:
            tray.setImage(TRAY_ICON_OFF)
            break
    }

    tray.setContextMenu(menu)
}
