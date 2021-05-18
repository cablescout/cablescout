const path = require('path')
const { app, Menu, Tray } = require('electron')
const log = require('electron-log')
const { CONFIG } = require('./config')
const { STATUS } = require('./status')
const { Tunnel } = require('./tunnel')

const TRAY_ICON_OFF = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.Off.Template.png')
const TRAY_ICON_PROGRESS = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.Progress.Template.png')
const TRAY_ICON_ON = path.join(__dirname, 'tray-icon', 'Cablescout.Tray.On.Template.png')

let tray = null

async function appWillQuit(event) {
  if (!STATUS.isConnected()) {
    return
  }
  event.preventDefault()
  const tunnel_name = STATUS.currTunnel()
  const tunnels = await CONFIG.getTunnels()
  const tunnel_config = tunnels[tunnel_name]
  if (tunnel_config) {
    await disconnectTunnel(tunnel_name, tunnel_config)
  }
  app.quit()
}

async function connectTunnel(name, config) {
  try {
    STATUS.setCurrTunnel(name)
    updateTray()
    const tunnel = new Tunnel(name, config)
    await tunnel.connect()
  } catch (err) {
    log.error(err)
    STATUS.setCurrTunnel(null)
  }
  updateTray()
}

async function disconnectTunnel(name, config) {
  const tunnel = new Tunnel(name, config)
  await tunnel.disconnect()
  STATUS.setCurrTunnel(null)
  updateTray()
}

async function updateTray() {
  const tunnels = await CONFIG.getTunnels()

  const tunnel_menu_items = Object.entries(tunnels).map(([name, tunnel_config]) => {
    const is_curr = STATUS.currTunnel() === name
    return {
      label: is_curr ? `Disconnect ${name}` : `Connect ${name}`,
      click: is_curr ? (() => disconnectTunnel(name, tunnel_config)) : (() => connectTunnel(name, tunnel_config))
    }
  })

  const menu = Menu.buildFromTemplate([
    ...tunnel_menu_items,
    {
      label: 'Add new tunnel...',
      enabled: false,
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

  if (!tray) {
    tray = new Tray(TRAY_ICON_OFF)

    if (process.platform === 'win32') {
      tray.on('click', tray.popUpContextMenu)
    }
  }

  tray.setImage(STATUS.currTunnel() ? (STATUS.isConnected() ? TRAY_ICON_ON : TRAY_ICON_PROGRESS) : TRAY_ICON_OFF)
  tray.setContextMenu(menu)
}

async function main() {
  await app.whenReady()

  if (app.dock) {
    app.dock.hide()
  }

  app.on('window-all-closed', event => event.preventDefault())
  app.on('will-quit', appWillQuit)

  await updateTray()
}

main()
