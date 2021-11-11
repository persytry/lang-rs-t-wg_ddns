
### introduce

This is use for wireguard vpn to auto restart it when dynamic domain is changed, that's ddns.  
I test it success at linux.

### use

```sh
wg_ddns -c /etc/wireguard/wg0.conf
```

You can execute wg_ddns directly, becauce '-c' option default value is '/etc/wireguard/wg0.conf'.

To generate and enable wg_ddns service: `wg_ddns -s`.
