interface Params {
    organisation: string;
    crate: string;
}

export function load({ params }: { params: Params }): App.PageData {
    return {
        title: `${params.organisation}/${params.crate}`,
    };
}
