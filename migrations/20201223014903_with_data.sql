-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";



drop table if exists VOTES;
drop table if exists CHOICE;
drop table if exists QUALITY;
drop table if exists USERS;
drop table if exists SIGN;

DROP TYPE IF EXISTS VALID_QUALITY_TYPES;
CREATE TYPE VALID_QUALITY_TYPES AS ENUM ('Percent', 'Binary', 'Multiple');

DROP TYPE IF EXISTS USER_EXTERNAL_IDP;
CREATE TYPE USER_EXTERNAL_IDP AS ENUM ('Google', 'Facebook');

DROP TYPE IF EXISTS QUALITY_DEFINED_BY;
CREATE TYPE QUALITY_DEFINED_BY AS ENUM (
    'System',
    'User'
);


create table SIGN (
    ID SERIAL,
    SIGN_NAME varchar not null unique,
    -- SIGN_START_DATE VARCHAR not null,
    -- SIGN_END_DATE VARCHAR not null,
    CREATED_AT timestamp not null default current_timestamp,
    UPDATED_AT timestamp not null default current_timestamp ,
    PRIMARY KEY(ID)
);

create table USERS (
    id uuid default uuid_generate_v4(),
    EXTERNAL_IDP_ID varchar not null,
    EXTERNAL_IDP USER_EXTERNAL_IDP NOT NULL,
    EMAIL varchar unique, -- removing "NOT NULL" because user may not share their email address
    DISPLAY_NAME varchar,
    SIGN_ID integer not null,
    
    -- Comment for ACTIVE fields below:
    -- user will come from google or facebook sign in
    -- in rare occassion, if we have to disable a user then we can use this ACTIVE option
    -- it is set to TRUE by default
    IS_ACTIVE boolean not null default true, 

    IS_INTERNAL boolean not null default false, 
    
    CREATED_AT timestamp not null default current_timestamp,
    UPDATED_AT timestamp not null default current_timestamp,
    PRIMARY KEY(ID)
);
ALTER TABLE USERS ADD CONSTRAINT USER_SIGN_ID_FKEY FOREIGN KEY (SIGN_ID) REFERENCES SIGN(ID) ON DELETE SET NULL;

create table QUALITY (
    id SERIAL,
    QUALITY_LONG VARCHAR NOT NULL,
    QUALITY_SHORT VARCHAR NOT NULL,
    QUALITY_TYPE VALID_QUALITY_TYPES default 'Binary' NOT NULL,
    
    -- OK TO LEAVE NULL. If BOTH SIGN_1_ID and SIGN_2_ID are NULL then it applies to ALL signs
    -- However, for USER DEFINED qualities, user has to provide at least one sign
    SIGN_1_ID integer,
    SIGN_2_ID integer,
    DEFINED_BY QUALITY_DEFINED_BY default 'System' NOT NULL,

    CREATED_AT timestamp not null default current_timestamp,
    UPDATED_AT timestamp not null default current_timestamp,
    CREATED_BY uuid NOT NULL,
    UPDATED_BY uuid NOT NULL,
    
    PRIMARY KEY(ID)
);
ALTER TABLE QUALITY ADD CONSTRAINT QUALITY_CREATED_ID_FKEY FOREIGN KEY (CREATED_BY) REFERENCES USERS(ID) ON DELETE SET NULL;
ALTER TABLE QUALITY ADD CONSTRAINT QUALITY_MODIFIED_BY_FKEY FOREIGN KEY (UPDATED_BY) REFERENCES USERS(ID) ON DELETE SET NULL;

create table CHOICE (
    ID SERIAL,
    QUALITY_ID integer not null,
    CHOICE_VALUE VARCHAR not null,
    CREATED_AT timestamp not null default current_timestamp,
    UPDATED_AT timestamp not null default current_timestamp ,
    CREATED_BY uuid,
    UPDATED_BY uuid,
    PRIMARY KEY(ID)    
);

ALTER TABLE CHOICE ADD CONSTRAINT CHOICE_QUALITY_ID_FKEY FOREIGN KEY (QUALITY_ID) REFERENCES QUALITY(ID) ON DELETE CASCADE;
ALTER TABLE CHOICE ADD CONSTRAINT CHOICE_CREATED_BY_FKEY FOREIGN KEY (CREATED_BY) REFERENCES USERS(ID) ON DELETE SET NULL;
ALTER TABLE CHOICE ADD CONSTRAINT CHOICE_UPDATED_BY_FKEY FOREIGN KEY (UPDATED_BY) REFERENCES USERS(ID) ON DELETE SET NULL;

create table VOTES (
    ID SERIAL,
    QUALITY_ID integer not null,
    CHOICE_ID integer not null,
    USER_ID uuid,
    CREATED_AT timestamp not null default current_timestamp,
    UPDATED_AT timestamp not null default current_timestamp ,
    PRIMARY KEY(ID)    
);

ALTER TABLE VOTES ADD CONSTRAINT VOTES_QUALITY_ID_FKEY FOREIGN KEY (QUALITY_ID) REFERENCES QUALITY(ID) ON DELETE SET NULL;
ALTER TABLE VOTES ADD CONSTRAINT VOTES_CHOICE_ID_FKEY FOREIGN KEY (CHOICE_ID) REFERENCES CHOICE(ID) ON DELETE SET NULL;
ALTER TABLE VOTES ADD CONSTRAINT VOTES_USER_ID_FKEY FOREIGN KEY (USER_ID) REFERENCES USERS(ID) ON DELETE SET NULL;

CREATE INDEX IDX_VOTES_Q_C_U ON VOTES (QUALITY_ID, CHOICE_ID, USER_ID);
CREATE INDEX IDX_VOTES_Q_C ON VOTES (QUALITY_ID, CHOICE_ID);


CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
   IF row(NEW.*) IS DISTINCT FROM row(OLD.*) THEN
      NEW.UPDATED_AT = now(); 
      RETURN NEW;
   ELSE
      RETURN OLD;
   END IF;
END;
$$ language 'plpgsql';


CREATE TRIGGER update_sign_modtime BEFORE UPDATE ON sign FOR EACH ROW EXECUTE PROCEDURE  update_updated_at_column();
CREATE TRIGGER update_users_modtime BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE  update_updated_at_column();
CREATE TRIGGER update_quality_modtime BEFORE UPDATE ON quality FOR EACH ROW EXECUTE PROCEDURE  update_updated_at_column();
CREATE TRIGGER update_choice_modtime BEFORE UPDATE ON choice FOR EACH ROW EXECUTE PROCEDURE  update_updated_at_column();
CREATE TRIGGER update_votes_modtime BEFORE UPDATE ON votes FOR EACH ROW EXECUTE PROCEDURE  update_updated_at_column();

