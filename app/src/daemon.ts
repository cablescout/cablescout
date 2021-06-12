import * as path from 'path'
import * as process from 'process'
import { stat, readFile } from 'fs/promises'
import * as log from 'electron-log'
import * as hasha from 'hasha'
import { runAsRoot } from './sudo'

export async function ensureDaemon(): Promise<void> {
    switch (process.platform) {
        case 'darwin':
            await installLaunchdDaemon()
            break
        default:
            throw new Error(`Unsupported platform ${process.platform}, please open an issue on GitHub to request support for this platform or run the daemon manually.`)
    }
}

const LABEL = 'io.cablescout'
const LAUNCHD_PLIST_FILE = `/Library/LaunchDaemons/${LABEL}.plist`

async function getDaemonPath(): Promise<string> {
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

async function installLaunchdDaemon(): Promise<void> {
    const daemon_path = await getDaemonPath()
    const daemon_hash = await await hasha.fromFile(daemon_path, { algorithm: 'sha256' })

    const plist = `\
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
    <dict>
        <key>Label</key>
        <string>${LABEL}</string>

        <key>Program</key>
        <string>${daemon_path}</string>

        <key>ProgramArguments</key>
        <array>
            <string>${daemon_path}</string>
            <string>--debug</string>
        </array>

        <key>EnvironmentVariables</key>
        <dict>
            <key>PATH</key>
            <string>/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
        </dict>

        <key>RunAtLoad</key>
        <true/>

        <key>KeepAlive</key>
        <true/>

        <key>StandardOutPath</key>
        <string>/var/log/cablescout-daemon.log</string>

        <key>StandardErrorPath</key>
        <string>/var/log/cablescout-daemon.log</string>

        <key>ExeHash</key>
        <string>${daemon_hash}</string>
    </dict>
</plist>
`

    log.debug(`[daemon] Checking if daemon is already installed`)
    try {
        const curr = await readFile(LAUNCHD_PLIST_FILE, { encoding: 'utf-8' })
        if (curr == plist) {
            log.debug(`[daemon] Daemon plist file is up to date`)
            return
        }
    } catch (err) {
        log.warn(`[daemon] While checking for existing daemon plist: ${err}`)
    }

    log.debug(`[daemon] Creating plist file and calling launchctl`)
    const command = [
        `echo ${Buffer.from(plist).toString('base64')} | base64 -d > ${LAUNCHD_PLIST_FILE}`,
        `(launchctl unload -w ${LAUNCHD_PLIST_FILE}) || true`,
        `launchctl load -w ${LAUNCHD_PLIST_FILE}`,
    ].join(' && ')
    await runAsRoot(command)
}