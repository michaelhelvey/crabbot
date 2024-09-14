import { ok as assert } from "node:assert";
import * as aws from "@pulumi/aws";
import * as awsx from "@pulumi/awsx";
import * as apigateway from "@pulumi/aws-apigateway";

const PROJECT_NAME = "crabbot";

const requireEnv = (envKey: string) => {
  const resolved = process.env[envKey as string];
  assert(resolved, `Expected process.env to contain ${envKey}`);
  return resolved;
};

// Create an ECR repository to store the Docker image
const repo = new aws.ecr.Repository("lambda-ecr-repo", {
  name: PROJECT_NAME,
  forceDelete: true,
});

const image = new awsx.ecr.Image("image", {
  repositoryUrl: repo.repositoryUrl,
  context: ".",
  platform: "linux/arm64",
  imageTag: "latest",
});

// Get the repository credentials to be used by Lambda
const lambdaRole = new aws.iam.Role("lambda-role", {
  assumeRolePolicy: aws.iam.assumeRolePolicyForPrincipal({ Service: "lambda.amazonaws.com" }),
});

// Attach the necessary policies to the Lambda execution role
new aws.iam.RolePolicyAttachment("lambda-managed-policy", {
  role: lambdaRole.name,
  policyArn: aws.iam.ManagedPolicies.AWSLambdaBasicExecutionRole,
});

// Create the Lambda function using the Docker image
const lambda = new aws.lambda.Function("docker-lambda", {
  name: PROJECT_NAME,
  packageType: "Image",
  imageUri: image.imageUri,
  role: lambdaRole.arn,
  architectures: ["arm64"],
  timeout: 10, // Set Lambda timeout
  environment: {
    variables: {
      DISCORD_APP_ID: requireEnv("DISCORD_APP_ID"),
      DISCORD_PUBLIC_KEY: requireEnv("DISCORD_PUBLIC_KEY"),
      DISCORD_BOT_TOKEN: requireEnv("DISCORD_BOT_TOKEN"),
      AWS_LWA_READINESS_CHECK_PATH: "/health",
      PORT: "8080", // default for lambda web adapter
    },
  },
});

// Create an API Gateway
const api = new apigateway.RestAPI("crabbot", {
  description: "crabbot api",
  stageName: "v1",
  routes: [
    {
      path: "/health",
      method: "GET",
      eventHandler: lambda,
    },
    {
      path: "/interactions",
      method: "POST",
      eventHandler: lambda,
    },
  ],
});

// Define usage plan for rate limiting
const usagePlan = new aws.apigateway.UsagePlan("usage-plan", {
  name: `${PROJECT_NAME} usage plan`,
  throttleSettings: {
    burstLimit: 10,
    rateLimit: 10,
  },
  apiStages: [
    {
      apiId: api.api.id,
      stage: api.stage.stageName,
    },
  ],
});

// Export the URL of the API Gateway
export const url = api.url;
