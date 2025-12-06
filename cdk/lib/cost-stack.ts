import * as cdk from 'aws-cdk-lib/core';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as events from 'aws-cdk-lib/aws-events';
import * as targets from 'aws-cdk-lib/aws-events-targets';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as path from "path";
import { Construct } from 'constructs';

export interface CostStackProps extends cdk.StackProps {
  schedule?: events.Schedule;
  discordWebhookUrl: string;
}

export class CostStack extends cdk.Stack {
  public readonly costNotifierFunction: lambda.Function;

  constructor(scope: Construct, id: string, props: CostStackProps) {
    super(scope, id, props);

    // Lambda関数用のIAMロール
    const role = new iam.Role(this, "CostNotifierRole", {
      assumedBy: new iam.ServicePrincipal("lambda.amazonaws.com"),
    });

    role.addManagedPolicy(
      iam.ManagedPolicy.fromAwsManagedPolicyName(
        "service-role/AWSLambdaBasicExecutionRole"
      )
    );

    role.addToPolicy(
      new iam.PolicyStatement({
        actions: [
          "ce:GetCostAndUsage",
          "ce:GetCostForecast",
          "organizations:ListAccounts",
          "organizations:DescribeAccount",
        ],
        resources: ["*"],
      })
    );

    const fn = new lambda.Function(this, "CostNotifier", {
      runtime: lambda.Runtime.PROVIDED_AL2023,
      handler: "bootstrap",
      architecture: lambda.Architecture.ARM_64,
      role,
      memorySize: 256,
      timeout: cdk.Duration.seconds(60),
      environment: {
        DISCORD_WEBHOOK_URL: props.discordWebhookUrl,
      },
      code: lambda.Code.fromAsset(
        path.join(__dirname, "../../lambdas/CostNotifier/target/lambda/CostNotifier/bootstrap.zip")
      ),
    });

    this.costNotifierFunction = fn;

    // EventBridge rule（default: 9:00 AM JST = 00:00 UTC）
    const schedule = props.schedule || events.Schedule.cron({
      minute: '0',
      hour: '0',
      day: '*',
      month: '*',
      year: '*',
    });

    const rule = new events.Rule(this, 'CostNotificationSchedule', {
      schedule: schedule,
      description: 'Trigger cost notification daily',
    });

    rule.addTarget(new targets.LambdaFunction(this.costNotifierFunction));

    // Outputs
    new cdk.CfnOutput(this, 'LambdaFunctionName', {
      value: this.costNotifierFunction.functionName,
      description: 'Name of the cost notifier Lambda function',
    });

    new cdk.CfnOutput(this, 'LambdaFunctionArn', {
      value: this.costNotifierFunction.functionArn,
      description: 'ARN of the cost notifier Lambda function',
    });
  }
}
