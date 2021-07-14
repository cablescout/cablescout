import * as path from 'path'
import * as log from 'electron-log'
import { stat } from 'fs/promises'

export async function getDaemonPath(): Promise<string> {
    let res: string
    const { CABLESCOUT_DAEMON_PATH } = process.env
    if (CABLESCOUT_DAEMON_PATH) {
        log.debug(`[daemon] Using daemon path from env: ${CABLESCOUT_DAEMON_PATH}`)
        res = CABLESCOUT_DAEMON_PATH
    } else {
        res = path.join(process.resourcesPath, 'cablescout-daemon')
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
