/*
 Navicat Premium Dump SQL

 Source Server         : localhost_5432
 Source Server Type    : PostgreSQL
 Source Server Version : 170005 (170005)
 Source Host           : localhost:5432
 Source Catalog        : web_socket
 Source Schema         : public

 Target Server Type    : PostgreSQL
 Target Server Version : 170005 (170005)
 File Encoding         : 65001

 Date: 31/01/2026 04:24:48
*/


-- ----------------------------
-- Sequence structure for user_id_seq
-- ----------------------------
DROP SEQUENCE IF EXISTS "public"."user_id_seq";
CREATE SEQUENCE "public"."user_id_seq" 
INCREMENT 1
MINVALUE  1
MAXVALUE 9223372036854775807
START 1
CACHE 1;

-- ----------------------------
-- Table structure for application_use
-- ----------------------------
DROP TABLE IF EXISTS "public"."application_use";
CREATE TABLE "public"."application_use" (
  "id" int8 NOT NULL,
  "app_id" varchar(255) COLLATE "pg_catalog"."default",
  "token" varchar(255) COLLATE "pg_catalog"."default",
  "app_auth_url" varchar(1000) COLLATE "pg_catalog"."default",
  "app_callback_message" varchar(1000) COLLATE "pg_catalog"."default"
)
;
COMMENT ON COLUMN "public"."application_use"."id" IS '用户';
COMMENT ON COLUMN "public"."application_use"."app_id" IS '应用ID';
COMMENT ON COLUMN "public"."application_use"."token" IS 'toekn';
COMMENT ON COLUMN "public"."application_use"."app_auth_url" IS '授权ID';
COMMENT ON COLUMN "public"."application_use"."app_callback_message" IS '消息回调地址';

-- ----------------------------
-- Records of application_use
-- ----------------------------
INSERT INTO "public"."application_use" VALUES (1, 'app', '123456', 'http://localhost:7004/app/ws/auth', 'http://localhost:7004/pc28/message');

-- ----------------------------
-- Table structure for user
-- ----------------------------
DROP TABLE IF EXISTS "public"."user";
CREATE TABLE "public"."user" (
  "id" int8 NOT NULL GENERATED ALWAYS AS IDENTITY (
INCREMENT 1
MINVALUE  1
MAXVALUE 9223372036854775807
START 1
CACHE 1
),
  "username" varchar(255) COLLATE "pg_catalog"."default",
  "password" varchar(255) COLLATE "pg_catalog"."default",
  "nickname" varchar(255) COLLATE "pg_catalog"."default",
  "email" varchar(255) COLLATE "pg_catalog"."default",
  "phone" varchar(255) COLLATE "pg_catalog"."default"
)
;
COMMENT ON COLUMN "public"."user"."password" IS '密码';
COMMENT ON COLUMN "public"."user"."nickname" IS '昵称';
COMMENT ON COLUMN "public"."user"."email" IS '电子邮箱';
COMMENT ON COLUMN "public"."user"."phone" IS '电话号码';

-- ----------------------------
-- Records of user
-- ----------------------------
INSERT INTO "public"."user" OVERRIDING SYSTEM VALUE VALUES (5, 'admin', '$argon2id$v=19$m=19456,t=2,p=1$UFwWGp3HFpKHmMQgelNiWQ$qjZ54T1XZW5sUs24vcnDKhm+PHiZ/u7OBJRhPCxP/jE', '管理员2', '1234564803354420.com', '123456789');

-- ----------------------------
-- Alter sequences owned by
-- ----------------------------
ALTER SEQUENCE "public"."user_id_seq"
OWNED BY "public"."user"."id";
SELECT setval('"public"."user_id_seq"', 5, true);

-- ----------------------------
-- Primary Key structure for table application_use
-- ----------------------------
ALTER TABLE "public"."application_use" ADD CONSTRAINT "applicaton_use_pk" PRIMARY KEY ("id");

-- ----------------------------
-- Auto increment value for user
-- ----------------------------
SELECT setval('"public"."user_id_seq"', 5, true);

-- ----------------------------
-- Primary Key structure for table user
-- ----------------------------
ALTER TABLE "public"."user" ADD CONSTRAINT "user_pk" PRIMARY KEY ("id");
