import log from 'electron-log'
import { BrowserWindow } from 'electron'

export function oauthLogin(login_url: string, finish_url: string): Promise<string> {
  return new Promise((resolve, reject) => {
    log.debug('[login] Opening login window')
    const win = new BrowserWindow({
      width: 450,
      height: 700,
      center: true,
      alwaysOnTop: true,
      titleBarStyle: 'hidden',
      webPreferences: {
        nodeIntegration: false,
      },
    })
    win.loadURL(login_url)

    // If window is closed before we can complete this
    // promise successfully, reject it.
    win.on('close', event => {
      log.warn('[login] Login window closed')
      reject(new Error('Login window closed'))
    })

    log.debug(`[login] Waiting for finish URL: ${finish_url}`)
    win.webContents.session.webRequest.onBeforeRequest(
      { urls: [ `${finish_url}*` ] },
      ({ url }) => {
        log.info('[login] Finish URL opened, getting auth code')
        const parsed_url = new URL(url)
        const code = parsed_url.searchParams.get('code')
        if (code) {
          resolve(code)
        } else {
          reject(new Error('No code found after a seemingly successful login'))
        }
        reject = () => null
        win.close()
      },
    )
  })
}
