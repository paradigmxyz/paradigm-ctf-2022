// SPDX-License-Identifier: UNLICENSED

// DO NOT RELEASE SOURCE FOR THIS CHAL IT'S A REVERSING CHALLENGE.
//
// Challenge author: stong (cts), Zellic Inc.
// Challenge prepared for Paradigm CTF 2022

pragma solidity 0.8.16;

library Yummy8 {
    function yummy4(uint8 a, uint8 b) internal pure returns (uint8) {
        uint8 res = 0;
        for (; b != 0; b >>= 1) {
            if ((b & 1) != 0)
                res ^= a;
            if ((a & 0x80) != 0)
                a = (a << 1) ^ 0x1b;
            else
                a <<= 1;
        }
        return res;
    }

    function yummy5(uint8[6][6] memory a, uint8[6][6] memory b) internal pure returns (bool) {
        for (uint i = 0; i < 6; i++) {
            for (uint j = 0; j < 6 ; j++) {
                if (a[i][j] != b[i][j])
                    return false;
            }
        }
        return true;
    }

    function yummy6(uint8[6][6] memory a, uint8[6][6] memory b) internal pure returns (uint8[6][6] memory) {
        for (uint i = 0; i < 6; i++)
            for (uint j = 0; j < 6 ; j++)
                a[i][j] ^= b[i][j];
        return a;
    }

    function yummy7(uint8[6][6] memory a, uint8[6][6] memory b) internal pure returns (uint8[6][6] memory) {
        uint8[6][6] memory result;
        for (uint i = 0; i < 6; i++)
            for (uint j = 0; j < 6; j++)
                for (uint p = 0; p < 6; p++)
                    result[i][j] ^= yummy4(a[i][p], b[p][j]);
        return result;
    }
}

contract Challenge {
    using Yummy8 for uint8;
    using Yummy8 for uint8[6][6];

    bool public solved;

    function check(bytes calldata flag) public {
        require (flag.length == 36+6, "invalid flag");
        require (flag[0] == "P" && flag[1] == "C" && flag[2] == "T" && flag[3] == "F" && flag[4] == "{", "invalid flag");
        require (flag[flag.length - 1] == "}", "invalid flag");

        uint8[6][6] memory yummy9;
        for (uint i = 0; i < 6; i++)
            for (uint j = 0; j < 6; j++)
                yummy9[i][j] = uint8(flag[5+i*6+j]);

        uint8[6][6] memory yummy1 = [
            [0xbe, 0x9a, 0xc2, 0x24, 0x7f, 0x4d],
            [0x59, 0xde, 0x3b, 0x61, 0x0a, 0x1a],
            [0xc8, 0x18, 0x96, 0x0e, 0x94, 0x4d],
            [0xe3, 0x64, 0x8c, 0x6d, 0x76, 0xfe],
            [0x16, 0xd1, 0x41, 0x8e, 0x0e, 0x50],
            [0xe7, 0x42, 0xa4, 0x87, 0x8e, 0x6b]
        ];
        uint8[6][6] memory yummy2 = [
            [0x23, 0xab, 0x1e, 0x4c, 0xe9, 0xe],
            [0xef, 0x53, 0xb4, 0xac, 0x18, 0xb1],
            [0x3c, 0xc2, 0x2f, 0x34, 0x4a, 0x18],
            [0x65, 0x94, 0x67, 0xd3, 0x59, 0x29],
            [0xa0, 0x27, 0x4a, 0x73, 0xcd, 0x88],
            [0x5e, 0x32, 0x50, 0x20, 0x80, 0xe]
        ];

        uint8[6][6] memory yummy3 = [
            [98, 98, 45, 87, 130, 42],
            [200, 184, 107, 102, 139, 25],
            [73, 139, 58, 136, 217, 129],
            [219, 21, 107, 204, 18, 219],
            [145, 192, 17, 86, 166, 217],
            [40, 248, 43, 71, 93, 226]
        ];

        // Despite multiple applications, this is still affine
        // y1^6*X + (y1^5*y2 + y1^4*y2 + y1^3*y2 + y1^2*y2 + y1*y2 + y2)
        // Can crack it blackbox with side channel like gas
        yummy9 = yummy2.yummy7(yummy9).yummy6(yummy1);
        yummy9 = yummy2.yummy7(yummy9).yummy6(yummy1);
        yummy9 = yummy2.yummy7(yummy9).yummy6(yummy1);
        yummy9 = yummy2.yummy7(yummy9).yummy6(yummy1);
        yummy9 = yummy2.yummy7(yummy9).yummy6(yummy1);
        yummy9 = yummy2.yummy7(yummy9).yummy6(yummy1);

        // NOT a constant time comparison.
        require(yummy9.yummy5(yummy3), "invalid flag");

        solved = true;
    }
}