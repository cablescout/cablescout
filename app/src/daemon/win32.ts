import { runAsRoot } from '../sudo'
import { getDaemonPath } from './utils'

export async function installWin32Service(): Promise<void> {
    const daemon = await getDaemonPath()
    runAsRoot(`sc create cablescout-daemon binPath= "${process.resourcesPath}/shawl.exe run -- ${daemon}" start= auto`)
}
