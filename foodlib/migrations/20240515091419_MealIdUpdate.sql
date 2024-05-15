alter table public.event_meals
    add id integer;

alter table public.event_meals
    drop constraint event_meals_pk;

alter table public.event_meals
    add constraint event_meals_pk
        primary key (id);
