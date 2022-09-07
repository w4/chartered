<script type="typescript">
    import { DateTime } from 'luxon';

    /**
     * A time string to convert to relative time, can be anything that can be read by
     * `Date.parse`.
     */
    export let time: string;

    const date = DateTime.fromMillis(Date.parse(time));
</script>

<time title={date.toLocaleString(DateTime.DATETIME_MED)} datetime={date.toISO()}>
    {date.toRelative()}
</time>
