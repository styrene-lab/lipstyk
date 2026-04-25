import axios from 'axios';

// fetch the data
async function fetchData(url: string): Promise<any> {
    const response = await axios.get(url);
    const data: any = response.data;
    return data;
}

// process the data
function processData(items: any[]): any[] {
    return items.map((item: any) => {
        const result: any = {
            name: item.name,
            value: item.value,
        };
        return result;
    });
}

// handle the request
async function handleRequest(req: any, res: any): Promise<void> {
    try {
        const data = await fetchData("https://api.example.com/data");
        const processed = processData(data as any);
        console.log("processed data:", processed);
        console.log("items count:", processed.length);
        console.log("first item:", processed[0]);
        res.json(processed);
    } catch (error) {
        console.error("Error:", error);
        console.log("Failed to process request");
        res.status(500).json({ error: "Internal server error" });
    }
}

// get the result
function getData(): Promise<object> {
    return fetch("/api/data")
        .then(res => res.json())
        .then(data => data.items)
        .then(items => items.filter((i: any) => i.active))
        .catch(() => {});
}

// @ts-ignore
const legacyValue = window.oldAPI.getValue();
// @ts-ignore
const anotherLegacy = window.oldAPI.process();
// @ts-expect-error
const brokenThing: number = "not a number";

const status = true;
const label = status ? "active" : status === false ? "inactive" : status === null ? "unknown" : "error";

// TODO: Add error handling
// TODO: Implement this
// TODO: Add validation
