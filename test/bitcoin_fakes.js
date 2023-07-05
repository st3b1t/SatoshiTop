
module.exports = {
    fake_blockchaininfo: () => {
        return {
          "chain": "main",
          "blocks": 795163,
          "headers": 795163,
          "bestblockhash": "00000000000000cf2a9b6b5d09ef182f5f6999a33ee0bb109d",
          "difficulty": 52350439455487.47,
          "time": 1687512682,
          "mediantime": 1687508210,
          "verificationprogress": 0.9999947507366781,
          "initialblockdownload": false,
          "chainwork": "000000000000000000000000000000a02ed9f896cc1db4",
          "size_on_disk": 556672030542,
          "pruned": false,
          "warnings": ""
        }
    },
    fake_networkinfo: () => {
        return {
            "version": 220000,
            "subversion": "/Satoshi:22.0.0/",
            "protocolversion": 70016,
            "localservices": "0000000000000409",
            "localservicesnames": [
                "NETWORK",
                "NETWORK_LIMITED"
            ],
            "localrelay": true,
            "timeoffset": 0,
            "networkactive": true,
            "connections": 41,
            "connections_in": 31,
            "connections_out": 10,
            "networks": [
                {
                    "name": "ipv4",
                    "limited": false,
                    "reachable": true,
                    "proxy": "",
                    "proxy_randomize_credentials": false
                },
                {
                    "name": "onion",
                    "limited": false,
                    "reachable": true,
                    "proxy": "127.0.0.1:9050",
                    "proxy_randomize_credentials": true
                }
            ],
            "relayfee": 0.00001,
            "incrementalfee": 0.00001,
            "localaddresses": [
                {
                    "address": "m72cepqoa2l54adxtjwb2x4ym67g3qwdd7ci6oddzcco5yjqd7mrdwqk.onion",
                    "port": 8333,
                    "score": 4
                }
            ],
            "warnings": ""
        }
    }
}