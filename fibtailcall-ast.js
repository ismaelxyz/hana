[
  {
    expr: {
      left: {
        callee: { identifier: "fibrec" },
        args: [
          { identifier: "n" },
          { identifier: "prev" },
          { identifier: "curr" },
        ],
        op: "call",
      },
      right: {
        cond: { left: { identifier: "n" }, right: { integer: 0 }, op: "<=" },
        then: { identifier: "curr" },
        alt: {
          callee: { identifier: "fibrec" },
          args: [
            { left: { identifier: "n" }, right: { integer: 1 }, op: "-" },
            {
              left: { identifier: "prev" },
              right: { identifier: "curr" },
              op: "+",
            },
            { identifier: "prev" },
          ],
          op: "call",
        },
        op: cond,
      },
      op: "=",
    },
    type: exprstmt,
  },
  {
    expr: {
      left: {
        callee: { identifier: "fib" },
        args: [{ identifier: "n" }],
        op: "call",
      },
      right: {
        callee: { identifier: "fibrec" },
        args: [
          { left: { identifier: "n" }, right: { integer: 1 }, op: "+" },
          { integer: 1 },
          { integer: 0 },
        ],
        op: "call",
      },
      op: "=",
    },
    type: exprstmt,
  },
  {
    expr: {
      callee: { identifier: "print" },
      args: [
        { callee: { identifier: "fib" }, args: [{ integer: 30 }], op: "call" },
        { string: "\n" },
      ],
      op: "call",
    },
    type: exprstmt,
  },
];
