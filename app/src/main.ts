import { app } from 'electron'
import log from 'electron-log'
import updateElectronApp from 'update-electron-app'
import { getStatus, disconnectTunnel } from './client'
import { ensureDaemon } from './daemon'
import { updateTray } from './tray'

async function appWillQuit(event: Event) {
    log.warn('[main] App about to quit')
    const status = await getStatus()
    if (!status.currentTunnel) {
        log.info('[main] No tunnel connected, quitting app')
        return
    }
    event.preventDefault()
    log.info(`[main] Disconnecting ${status.currentTunnel.name} before quitting app`)
    await disconnectTunnel()
    app.quit()
}

async function main() {
    log.info('[main] =================== Starting app ===================')

    if (require('electron-squirrel-startup')) {
        return app.quit()
    }

    updateElectronApp({
        logger: log,
    })

    await app.whenReady()
    log.debug('[main] App ready')

    if (app.dock) {
        app.dock.hide()
    }

    app.on('window-all-closed', (event: Event) => event.preventDefault())
    app.on('will-quit', appWillQuit)

    await ensureDaemon()
    await updateTray()
}

log.catchErrors({ showDialog: true })
main()
