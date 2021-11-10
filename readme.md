
### introduce

this is use for wireguard vpn to auto restart it when dynamic domain is changed, that's ddns.  
I test it success at linux.

### use

```sh
wg_ddns -c /etc/wireguard/wg0.conf
```

You can execute wg_ddnx directly, becauce '-c' option default value is '/etc/wireguard/wg0.conf'.

