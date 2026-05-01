This directory contains avif test files for CrabbyAvif

## white_iden_irot.avif

- Item 1: AV1 item.
- Item 2 (primary): Identity item with irot property pointing to Item 1.

Created with:
```bash
$ MP4Box -add-derived-image :type=iden:ref=dimg,1:rotation=90:image-size=2x2 white_2x2.avif -out white_iden_irot.avif

$ MP4Box -set-primary 2 white_iden_irot.avif
```

## white_iden_chain_with_imir.avif

- Item 1: AV1 item.
- Item 2: Identity item with imir property pointing to Item 1.
- Item 3: Identity item pointing to Item 2.
- Item 4 (primary): Identity item pointing to Item 3.

Created with:
```bash
$ MP4Box -add-derived-image :type=iden:ref=dimg,1:mirror-axis=vertical:image-size=2x2 -add-derived-image :type=iden:ref=dimg,2:image-size=2x2 -add-derived-image :type=iden:ref=dimg,3:image-size=2x2 white_2x2.avif -out white_iden_chain_with_imir.avif

$ MP4Box -set-primary 4 white_iden_chain_with_imir.avif
```

## white_iden_chain_with_imir_primary.avif

- Item 1: AV1 item.
- Item 2: Identity item pointing to Item 1.
- Item 3: Identity item pointing to Item 2.
- Item 4 (primary): Identity item with imir property pointing to Item 3.

Created with:
```bash
$ MP4Box -add-derived-image :type=iden:ref=dimg,1:image-size=2x2 -add-derived-image :type=iden:ref=dimg,2:image-size=2x2 -add-derived-image :type=iden:ref=dimg,3:mirror-axis=vertical:image-size=2x2 white_2x2.avif -out white_iden_chain_with_imir_primary.avif

$ MP4Box -set-primary 4 white_iden_chain_with_imir_primary.avif
```

## white_iden_cycle.avif

- Item 1: AV1 item.
- Item 2: Identity item with imir property pointing to Item 4.
- Item 3: Identity item pointing to Item 2.
- Item 4 (primary): Identity item pointing to Item 3.

Created with:
```bash
$ MP4Box -add-derived-image :type=iden:ref=dimg,4:image-size=2x2 -add-derived-image :type=iden:ref=dimg,2:image-size=2x2 -add-derived-image :type=iden:ref=dimg,3:image-size=2x2 white_2x2.avif -out white_iden_cycle.avif

$ MP4Box -set-primary 4 white_iden_cycle.avif
```

## white_iden_self.avif

- Item 1: AV1 item.
- Item 2: Identity item pointing to Item 2..

Created with:
```bash
$ MP4Box -add-derived-image :type=iden:ref=dimg,2:image-size=2x2 -add-derived-image white_2x2.avif -out white_iden_self.avif

$ MP4Box -set-primary 2 white_iden_self.avif
```
