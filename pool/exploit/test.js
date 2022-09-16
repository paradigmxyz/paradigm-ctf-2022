var AMT = 10;

states = [[AMT, 0]];
tot = new Set();
var best = 0;
var bestVal = "";

var trans = {};

let process = (curr, nxt, amt, dir) => {
    if (!tot.has(nxt.join())) {
        tot.add(nxt.join());
        states.push(nxt);

        trans[nxt.join()] = [curr, amt, dir];

        var val = nxt[0] + nxt[1];
        if (val > best) {
            best = val;
            bestVal = nxt.join();
        }
    }
}

tot.add(states[0].join());

for (var i = 0; i < 100000; i++) {
    var curr = states.shift();
    if (curr === undefined) break;
    for (var amt = 1; amt < 110; amt++) {
        var x = 2 * AMT - curr[0];
        var y = AMT - curr[1];
        if (amt <= curr[0]) {
            var y1 = Math.floor(x * y / (x + amt));
            var x1 = x + amt;

            var nxt = [2 * AMT - x1, AMT - y1];
            process(curr, nxt, amt, true);

        }
        if (amt <= curr[1]) {
            var x1 = Math.floor(x * y / (y + amt));
            var y1 = y + amt;

            var nxt = [2 * AMT - x1, AMT - y1];
            process(curr, nxt, amt, false);
        }
    }
}

var vals = [];

var curr = bestVal;
console.log(bestVal);
for (let i = 0; i < 100; i++) {
  if (trans[curr] == undefined) break;
    let [nxt, amt, dir] = trans[curr];

    console.log(nxt, amt, dir);

    curr = nxt.join();

    vals.unshift(amt * (dir ? -1: 1));
}

console.log(vals.join(", "));
