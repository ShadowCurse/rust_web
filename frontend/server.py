#!/bin/python

import http.server, ssl, os

os.chdir("./dist")

server_address = ('0.0.0.0', 443)
httpd = http.server.HTTPServer(server_address, http.server.SimpleHTTPRequestHandler)
httpd.socket = ssl.wrap_socket(httpd.socket,
                               server_side=True,
                               certfile='../localhost.pem',
                               ssl_version=ssl.PROTOCOL_TLS)
print(f"serving server on: https://{server_address[0]}:{server_address[1]}")
httpd.serve_forever()