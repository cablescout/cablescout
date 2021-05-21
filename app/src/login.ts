import { BrowserWindow } from 'electron'

export function oauthLogin(login_url: string, finish_url: string): Promise<string | null> {
  return new Promise((resolve, reject) => {
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
    win.on('close', reject)

    win.webContents.session.webRequest.onBeforeRequest(
      { urls: [ `${finish_url}*` ] },
      ({ url }) => {
        const parsed_url = new URL(url)
        const code = parsed_url.searchParams.get('code')
        resolve(code)
        reject = () => null
        win.close()
      },
    )
  })
}
