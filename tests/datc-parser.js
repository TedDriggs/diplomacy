let pattern = /^6\.([A-Z])\.(\d+)\. TEST CASE, (.+)$/;

let rewriteTestCase = (text) => {
    const [, code, id, title] = text.match(pattern);
    return `t6${code.toLowerCase()}${id.toLowerCase()}_${title
        .replace(/[',?]/g, '')
        .replace(/[\s\-]/g, "_")
        .toLowerCase()}`;
};

/**
 *
 * @param {string} text
 */
let rewriteOrderBlock = (text) => {
    const [power, orderBlock] = text?.split(":") ?? [];
    const powerCode = power.substr(0, 3).toUpperCase();
    const orders = orderBlock
        ?.split("\n")
        .filter(l => l.trim() !== "")
        .map((order) => `${powerCode}: ${order.replace("-", "->")}`);
    return orders?.join("\n");
};

let rewriteOrderText = (text) => {
    return text.split('\n\n').map(rewriteOrderBlock).join('\n');
}

let handleNodes = (nodes) => {
    const found = [];
    for (let i = 0; i < nodes.length; i++) {
        const node = nodes[i];
        if (
            node.localName === "a" &&
            node.name?.startsWith("6") &&
            pattern.test(node.innerText)
        ) {
            const testCaseName = rewriteTestCase(node.innerText);
            const testCaseBody = rewriteOrderText(nodes[++i].innerText);
            found.push({
                name: testCaseName,
                body: testCaseBody,
            });
        }
    }

    return found;
};

let asRustTest = ({ name, body }) => {
    return [
        `#[test]`,
        `fn ${name}() {`,
        `let results = get_results(vec![${body
            ?.split("\n")
            .map((l) => `"${l}"`)
            .join(",")}]);`,
        `}`,
    ].join("\n");
};

handleNodes([...document.querySelectorAll('h4 a[name^="6"], pre')])
    .map(asRustTest)
    .join("\n\n");
