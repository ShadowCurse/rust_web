#!/bin/python

import http.server
import os
import ssl
import sys

if len(sys.argv) == 2:
    os.chdir("./testing")
else:
    os.chdir("./dist")

server_address = ('0.0.0.0', 443)
httpd = http.server.HTTPServer(server_address, http.server.SimpleHTTPRequestHandler)
httpd.socket = ssl.wrap_socket(httpd.socket,
                               server_side=True,
                               certfile='../cert.crt',
                               keyfile='../key.rsa',
                               ssl_version=ssl.PROTOCOL_TLS)
print(f"serving server on: https://{server_address[0]}:{server_address[1]}")
httpd.serve_forever()
