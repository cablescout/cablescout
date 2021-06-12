import * as log from 'electron-log'
import * as sudo from 'sudo-prompt'

export function runAsRoot(command: string): Promise<string> {
    log.debug(`[daemon] Running as root: ${command}`)
    return new Promise((resolve, reject) => {
        sudo.exec(
            command,
            { name: 'Cablescout' },
            (error?: Error | undefined, stdout?: string | Buffer | undefined, stderr?: string | Buffer | undefined) => {
                if (error) {
                    reject(`Error running command: ${error}:\n${stdout}\n${stderr}`)
                    return
                }
                let output: string
                if (!stdout) {
                    output = ''
                } else if (stdout instanceof Buffer) {
                    output = stdout.toString()
                } else {
                    output = stdout
                }
                log.debug(`[daemon] Output: ${output}`)
                resolve(output)
            },
        )
    })
}
