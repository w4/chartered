<script type="typescript">
    import { DateTime } from 'luxon';

    export let time: string;

    const date = DateTime.fromMillis(Date.parse(time));
</script>

<time title={date.toLocaleString(DateTime.DATETIME_MED)} datetime={date.toISO()}>
    {date.toRelative()}
</time>
