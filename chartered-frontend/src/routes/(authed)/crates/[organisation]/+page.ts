interface Params {
    organisation: string;
}

export function load({ params }: { params: Params }): App.PageData {
    return {
        title: params.organisation,
    };
}
