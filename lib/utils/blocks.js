
function avgBlockTime(blocks) {
    let prevTime = 0;
    let timeDiffs = [];

    // Loop through the last numBlocks to calculate time differences
    blocks.forEach(block => {
        if (prevTime !== 0) {
            const diff = block.time - prevTime;
            timeDiffs.push(diff);
        }
        prevTime = block.time;
    });

    // Calculate the average time difference
    const total = timeDiffs.reduce((acc, val) => acc + val, 0)
        , avgTime = total / timeDiffs.length;

    return avgTime;
}

module.exports = {
    avgBlockTime
}