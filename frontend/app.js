const express = require('express')
const { ToadScheduler, SimpleIntervalJob, Task } = require('toad-scheduler')
const fs = require('fs')

function string_number_to_bool(string) {
    const num = Number(string)
    return !!num
}

function get_codInfo_value(key, codInfo, bool = false, int = false) {
    const key_index = codInfo.indexOf(key)
    if (key_index !== -1 && codInfo !== '' && codInfo.includes(key)) {
        let keyval = codInfo
            .substring(key_index + key.length + 1, codInfo.length)
        if (keyval.includes('\\')) {
            // keyval still contains more keys, so split them off
            keyval = keyval.substring(0, keyval.indexOf('\\'))
        }
        if (bool) {
            return string_number_to_bool(keyval)
        }
        return keyval
    }
    if (int === true) {
        return 0
    }
    return ''
}

function parse_codInfo(codInfo) {
    const codInfo_parsed = {
        challenge: get_codInfo_value('challenge', codInfo),
        checksum: get_codInfo_value('checksum', codInfo),
        isPrivate: get_codInfo_value('isPrivate', codInfo, true),
        hostname: get_codInfo_value('hostname', codInfo),
        gamename: get_codInfo_value('gamename', codInfo),
        sv_maxclients: get_codInfo_value('sv_maxclients', codInfo, false, true),
        gametype: get_codInfo_value('gametype', codInfo),
        sv_motd: get_codInfo_value('sv_motd', codInfo),
        xuid: get_codInfo_value('xuid', codInfo),
        mapname: get_codInfo_value('mapname', codInfo),
        clients: get_codInfo_value('clients', codInfo, false, true),
        bots: get_codInfo_value('bots', codInfo, false, true),
        protocol: get_codInfo_value('protocol', codInfo),
        fs_game: get_codInfo_value('fs_game', codInfo),
        hc: get_codInfo_value('hc', codInfo, true),
        securityLevel: get_codInfo_value('securityLevel', codInfo),
        shortversion: get_codInfo_value('shortversion', codInfo),
        sv_running: get_codInfo_value('sv_running', codInfo, true),
        wwwDownload: get_codInfo_value('wwwDownload', codInfo),
        wwwUrl: get_codInfo_value('wwwUrl', codInfo),
    }

    return codInfo_parsed
}

let servers = []

function requireUncached(module) {
    delete require.cache[require.resolve(module)];
    return require(module);
}

const scheduler = new ToadScheduler()
const get_servers = new Task('get_servers', () => {
    let servers_parsed = []

    const files = fs.readdirSync('../backend/').filter(fn => fn.startsWith('servers_') && fn.endsWith('.json'))
    files.forEach(file => {
        const servers_json = requireUncached(`../backend/${file}`)
        
        servers_json.forEach(server => {
            let server_parsed = parse_codInfo(server.codInfo)
            server_parsed.ip = server.ip
            server_parsed.port = server.port
            server_parsed.game = server.game
            servers_parsed.push(server_parsed)
        })
    })

    servers = servers_parsed
})

const get_servers_job = new SimpleIntervalJob({ seconds: 20 }, get_servers)
scheduler.addSimpleIntervalJob(get_servers_job)
get_servers.execute()


const app = express()
app.disable("x-powered-by")

app.get(['/api/servers', '/api/servers/:game'], (req, res) => {
    let _servers
    if (req.params.game) {
        _servers = servers.filter(server => server.game.toLowerCase() === req.params.game.toLowerCase())
    } else {
        _servers = servers
    }

    res.json(_servers)
})

app.listen(8432)