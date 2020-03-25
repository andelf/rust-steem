# rust-steem

开个天坑吧

## city hash modification

```cpp
static uint128 CityMurmur(const char *s, size_t len, uint128 seed) {
  /*
  uint64 a = Uint128Low64(seed);
  uint64 b = Uint128High64(seed);
  */
  uint64 a = Uint128High64(seed);
  uint64 b = Uint128Low64(seed);
  // ....
}
```

```cpp
static uint64 HashLen16(uint64 u, uint64 v) {
  //   return Hash128to64(uint128(u, v));
  return Hash128to64(uint128(v, u));
}
```

## variant encoding

little endian

0x9  \t  => variant
0xXX length of Key
0x02 u64
0x05 string, with length prefix
ip reversed bytes