#BARFile Format

```
BARFile Format
(Unless specified, all data encoded in little-endian (VAX/IMB) format)

<bar file> ::= <file header> <book index> <books> <end of file>

<file header> ::= [u8;16] = <leader> <major version> <minor version> <number of books> 
                  <bible version>

<leader> ::= [u8;3] = b'B' b'A' b'R'

<major version> ::= u8 (eg. 0x01)

<minor version> ::= u8 (eg. 0x00)

<number of books> ::= u8 (eg. 66) = NBOOKS

<bible version> ::= [u8;10] (Zero padded ASCII - eg "NIV\0\0\0\0\0\0\0")


<book index> ::= NBOOKS * <book index entry>

<book index entry> ::= [u8;5] = <book number> <book offset>

<book number> ::= u8 (1 = Genesis, 66 = Revelation)

<book offset> ::= [u8;4] = u32 LE (offset of book from start of file)


<books> ::= NBOOKS * <book entry>

<book entry> ::= <book header> <chapter index> <book data>

<book header> ::= <book number> <number of chapters>

<chapter index> ::= NCHAPT * <chapter index entry>

<chapter index entry> ::= [u8;4] = u32 LE (offset of chapter from start of book) 

<book data> ::= <data block> <book data> 
                | <end of block>


<data block> ::= <v1 block info> <LZO compressed data>
                | <v2 block info> <compressed data>


<v1 block info> ::= [u8;6] = <chapter number> + <start verse> + <end verse> + <block size>

<v2 block info> ::= [u8;7] = <chapter number> + <start verse> + <end verse> + 
                    <compression> + <block size>

<chapter number> ::= u8 (1 is 1st)

<start verse> ::= u8 (1 is 1st. Some Psalms have a bit of prelude text that goes in 0)

<end verse> ::= u8 (1 is 1st)

<compression> ::= u8 (0 = None, 1 = LZO, 2 = ZLib, 3 = GZip. v1 is always LZO)

<block size> ::= [u8;4] = u32 LE (size in bytes of compressed data block that follows) = BLOCKSIZE

<compressed data> ::= <plain text>
                     | <LZO compressed data>
                     | <ZLib compressed data>
                     | <GZip compressed data>

<LZO compressed data> ::= <LZO leader> <LZO uncompressed size> <LZO bytes>


(block info):=
00    BYTE      chapter number (1 is first)
01    BYTE      start verse    (1 is first)
02    BYTE      end verse      (1 is first)
<version 2.0>
03    BYTE      compression algorithm  1: LZO, 2: ZLIB, 3: GZIP
04-07 LONG      block size                     {BSIZE}
</version 2.0>
<version 1.0>
03-06 LONG      block size                           {BSIZE}
</version 1.0>

(compressed data):=
   BSIZE x BYTE  LZO compressed data        


   compressed data uncompresses to be whole number of verses separated by 
   newlines.

(end of book):=
00    BYTE      end of book byte = 0x00

(end of file):=
00    BYTE      end of file byte = 0x00
```