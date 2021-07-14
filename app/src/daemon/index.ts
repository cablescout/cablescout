import * as process from 'process'
import { installDarwinDaemon } from './darwin'
import { installWin32Service } from './win32'

export async function ensureDaemon(): Promise<void> {
    switch (process.platform) {
        case 'darwin':
            return await installDarwinDaemon()
        case 'win32':
            return await installWin32Service()
        default:
            throw new Error(`Unsupported platform ${process.platform}, please open an issue on GitHub to request support for this platform or run the daemon manually.`)
    }
}
