# Power Clash

Finds matching solidity function signatures for a given 4 byte signature hash and arguments. Useful for finding collisions or 0x00000000 gas saving methods (though there are better techniques for saving gas on calldata)

## Build

`cargo build`

then `./power-clash -h`

or `docker build . -t power-clash`

then `docker run --rm power-clash -h`

## Usage

Example:

```
$ docker run --rm power-clash -a address,address,bytes -s fa461e33 -p Test
Attempting to find Test*****(address,address,bytes) match for 0xfa461e33 in 19770609664 max permutations
FOUND match in 53.342570899s
TestBrMSja(address,address,bytes) should match 0xfa461e33
```

```
USAGE:
    power-clash [OPTIONS] --arg-signature <ARG_SIGNATURE> --prefix <PREFIX>

OPTIONS:
    -a, --arg-signature <ARG_SIGNATURE>
            Arguments string from the target function signature. Ex: addres,address,bytes

    -c, --char-set <CHAR_SET>
            Character set to use for random string [default:
            abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ]

    -h, --help
            Print help information

    -p, --prefix <PREFIX>
            Method name prefix before random string. Ex: LolSwap (for computed
            LolSwapAd75(address,address,bytes))

    -r, --rnd-len <RND_LEN>
            Length of random string [default: 6]

    -s, --sighash <SIGHASH>
            Target 4 byte signature hash. Ex: fa461e33 [default: 00000000]

    -V, --version
            Print version information
```
