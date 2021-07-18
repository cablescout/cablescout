import * as hasha from 'hasha'
import * as log from 'electron-log'
import { runAsRoot } from '../sudo'
import { getDaemonPath } from './utils'

const DAEMON_NAME = 'CableScoutDaemon'

export async function installWin32Service(): Promise<void> {
    const daemon_path = await getDaemonPath()
    const daemon_hash = await hasha.fromFile(daemon_path, { algorithm: 'sha256' })
    const cmdline = `${process.resourcesPath}/shawl.exe run --name ${DAEMON_NAME} -- ${daemon_path} --hash=${daemon_hash}`
    const current_cmdline = await currentWin32ServiceCmdline()

    log.debug(`Current cmdline  : ${current_cmdline}`)
    log.debug(`Expected cmdline : ${cmdline}`)

    if (current_cmdline == cmdline) {
        log.info('Service is up to date')
        return
    }

    await uninstallWin32Service()
    runAsRoot(`${process.resourcesPath}/shawl.exe add --name ${DAEMON_NAME} -- ${daemon_path} && sc config ${DAEMON_NAME} start= auto && sc start ${DAEMON_NAME}`)
}

async function currentWin32ServiceCmdline(): Promise<string> {
    try {
        const config = await runAsRoot(`sc qc ${DAEMON_NAME}`)
        const cmdline = config.match(/^\s*BINARY_PATH_NAME\s*:\s*(.+)$/m)
        if (cmdline) {
            return cmdline[1]
        }
    } catch (err) {
        log.debug(`Ignoring error while trying to get current service command line: ${err}`)
    }
    return ''
}

async function uninstallWin32Service(): Promise<void> {
    try {
        runAsRoot(`sc stop ${DAEMON_NAME} & sc delete ${DAEMON_NAME}`)
    } catch (err) {
        log.debug(`Ignoring error while checking and removing existing service: ${err}`)
    }
}
