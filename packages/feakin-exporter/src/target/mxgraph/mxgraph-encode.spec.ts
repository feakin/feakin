import MxGraphEncode from './mxgraph-encode';
import { MXCell, MxFileRoot, MxGraph } from './mxgraph';
import * as fs from "fs";
import { js2xml } from "./xml-converter";

describe('MxGraphEncoder', () => {
  it('should work', () => {
    const sampleStr =
      '<mxfile host="Electron" modified="2022-07-26T06:50:38.916Z" agent="5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) draw.io/19.0.3 Chrome/102.0.5005.63 Electron/19.0.3 Safari/537.36" etag="S-i-T2jFARcjY2b3OQvQ" version="19.0.3" type="device"><diagram id="bN4dli3-ZhfTxRglmdzi" name="Page-1">jZJNT8QgEIZ/DUcTWlbdPdp11Rj1YA97NKSMhQRKw1Lb+uulduhHNpvsieGZD2begbC96Z4dr+W7FaBJSkVH2CNJ0yTdbcMxkH4k2106gtIpgUEzyNUvIKRIGyXgtAr01mqv6jUsbFVB4VeMO2fbddi31etXa17CGcgLrs/pUQkvcYpbOvMXUKWMLycUPYbHYAQnyYVtF4gdCNs7a/1omW4PehAv6jLmPV3wTo05qPw1CQ+feUk39r4zx+yL5R/ZK7zdYJUfrhscGJv1fVTA2aYSMBShhGWtVB7ymheDtw07D0x6o8MtCSaWA+ehu9hnMk0fvg1YA971IQQT2B0Khj9mw/DezvonUVS50D7mcVx5OZWeVQkGChOv8wL+fYtvzA5/</diagram></mxfile>';
    const data = MxGraphEncode.decodeXml(sampleStr);
    expect(data?.startsWith('<mxGraphModel')).toBeTruthy();
  });

  it('encode with decode', () => {
    const source = '<mxGraphModel><root><mxCell id="0"/></root></mxGraphModel>';
    const encoded = MxGraphEncode.encode(source);
    const decoded = MxGraphEncode.decode(encoded);
    expect(decoded).toEqual(source);
  });

  it('text from hello world', () => {
    const sampleStr =
      '<mxfile host="Electron" modified="2022-07-26T12:42:39.225Z" agent="5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) draw.io/19.0.3 Chrome/102.0.5005.63 Electron/19.0.3 Safari/537.36" etag="O1qmAbmb6dEBzrUx5KlV" version="19.0.3" type="device"><diagram id="bN4dli3-ZhfTxRglmdzi" name="Page-1">jZNNT4QwEIZ/i4ceTYDuh1cX190Y9SCHPZqGjrRJYbAUYf31lm270BgTL2T6vPPRzgyE5vV40KwVL8hBkSzhI6EPJMvWq9R+J3B2gG4zByotuUPpDAr5DR4mnvaSQxc5GkRlZBvDEpsGShMxpjUOsdsHqrhqyyr4BYqSqd/0JLkRjt6tk5kfQVYiVE4Tr9QsOHvQCcZxWCC6JzTXiMZZ9ZiDmnoX+uLiHv9QrxfT0Jj/BNy/FVWywu1Yn3bvtHjdPcHzrc/yxVTvH0yyjbL5dq0Ga1aTGVDXsmZqtWKd6+rms8eLMJsh5miLI8ly6z+gVvwmJLEXdHni3BZHFS8dM+cwBo19w2F6SWLlQUgDRcvKSR3s3lkmTK3sKb1GLzsTngnawLhAvlMHwBqMPlsXr9KNn5pf2xX152FegjRMViwWIMQxv3fVNfU8Gmv46YTjvAUXbfEr0f0P</diagram></mxfile>';
    const encoded = MxGraphEncode.decodeXml(sampleStr);

    const xmlInJson: MxGraph | any = MxGraphEncode.xml2obj(encoded!);
    const mxCell: MXCell[] = xmlInJson.mxGraphModel.root.mxCell;

    expect(mxCell.length).toBe(3);
    expect(mxCell[2].attributes?.value?.includes('Hello, world!')).toBeTruthy();
  });

  it('xml to object', () => {
    const sampleStr =
      '<mxfile host="Electron" modified="2022-07-26T12:42:39.225Z" agent="5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) draw.io/19.0.3 Chrome/102.0.5005.63 Electron/19.0.3 Safari/537.36" etag="O1qmAbmb6dEBzrUx5KlV" version="19.0.3" type="device"><diagram id="bN4dli3-ZhfTxRglmdzi" name="Page-1">jZNNT4QwEIZ/i4ceTYDuh1cX190Y9SCHPZqGjrRJYbAUYf31lm270BgTL2T6vPPRzgyE5vV40KwVL8hBkSzhI6EPJMvWq9R+J3B2gG4zByotuUPpDAr5DR4mnvaSQxc5GkRlZBvDEpsGShMxpjUOsdsHqrhqyyr4BYqSqd/0JLkRjt6tk5kfQVYiVE4Tr9QsOHvQCcZxWCC6JzTXiMZZ9ZiDmnoX+uLiHv9QrxfT0Jj/BNy/FVWywu1Yn3bvtHjdPcHzrc/yxVTvH0yyjbL5dq0Ga1aTGVDXsmZqtWKd6+rms8eLMJsh5miLI8ly6z+gVvwmJLEXdHni3BZHFS8dM+cwBo19w2F6SWLlQUgDRcvKSR3s3lkmTK3sKb1GLzsTngnawLhAvlMHwBqMPlsXr9KNn5pf2xX152FegjRMViwWIMQxv3fVNfU8Gmv46YTjvAUXbfEr0f0P</diagram></mxfile>';

    const xmlInJson: MxGraph | any = MxGraphEncode.xml2obj(sampleStr!);
    console.log(JSON.stringify(xmlInJson, null, 4));
  });

  it('convert model to file', () => {
    const cells: MXCell[] = [
      {
        attributes: {
          id: "0",
        }
      },
      {
        attributes: {
          id: "ARSg04o7xmWB_3SNBJeL-0",
          vertex: 1,
          style: "rounded=0;whiteSpace=wrap;html=1;",
          value: "Hello, world!",
          parent: "0",
        },
        mxGeometry: {
          attributes: {
            as: "geometry",
            width: 100,
            height: 60
          }
        }
      }
    ];

    const graph: MxGraph = {
      mxGraphModel: {
        root: {
          mxCell: cells
        }
      }
    };

    const file: MxFileRoot = {
      mxfile: {
        diagram: {
          _text: MxGraphEncode.encode(js2xml(graph))
        }
      }
    }

    const drawioFile = js2xml(file);
    fs.writeFileSync('./test.drawio', drawioFile);

    const data = MxGraphEncode.decodeXml(drawioFile);
    const xmlInJson: MxGraph | any = MxGraphEncode.xml2obj(data!);
    const mxCell: MXCell[] = xmlInJson.mxGraphModel.root.mxCell;

    expect(mxCell.length).toBe(2);
  });
});
