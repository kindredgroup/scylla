CREATE TABLE IF NOT EXISTS public.task
(
    data jsonb NOT NULL
);


CREATE UNIQUE INDEX IF NOT EXISTS task_data_rn_idx
    ON public.task USING btree
    ((data ->> 'rn'::text) ASC NULLS LAST);
