import * as path from 'path'
import * as process from 'process'
import * as log from 'electron-log'
import { stat } from 'fs/promises'

const DAEMON_BASENAME = process.platform === 'win32' ? 'cablescout-daemon.exe' : 'cablescout-daemon'

export async function getDaemonPath(): Promise<string> {
    let res: string
    const { CABLESCOUT_DAEMON_PATH } = process.env
    if (CABLESCOUT_DAEMON_PATH) {
        log.debug(`[daemon] Using daemon path from env: ${CABLESCOUT_DAEMON_PATH}`)
        res = CABLESCOUT_DAEMON_PATH
    } else {
        res = path.join(process.resourcesPath, DAEMON_BASENAME)
        log.debug(`[daemon] Daemon path: ${CABLESCOUT_DAEMON_PATH}`)
    }
    try {
        await stat(res)
        return res
    } catch (err) {
        log.error(`[daemon] While searching for daemon path: ${err}`)
        throw new Error(`Could not find daemon executable: ${err}`)
    }
}
